//! Windows sensors: PDH performance counters and D3DKMT adapter queries. No admin rights needed.
//! Adapted from islet (https://github.com/joezhuo2/islet, MIT).

use windows_sys::Win32::System::Performance::{
    PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY, PDH_MORE_DATA,
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW,
};

use super::types::GpuKind;

/// A PDH query with any number of (usually wildcard) counters, collected together.
pub struct Query {
    query: PDH_HQUERY,
    counters: Vec<Option<PDH_HCOUNTER>>,
}

// The handles are only used from the collector thread.
unsafe impl Send for Query {}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

impl Query {
    /// Opens a query and adds `paths`. A path that fails to add (counter missing on this machine)
    /// yields `None` from [`Query::items`] instead of failing the whole query.
    pub fn new(paths: &[&str]) -> Option<Self> {
        // SAFETY: out-pointers are valid locals; the query is closed in Drop.
        unsafe {
            let mut query: PDH_HQUERY = std::mem::zeroed();
            if PdhOpenQueryW(std::ptr::null(), 0, &mut query) != 0 {
                return None;
            }
            let counters = paths
                .iter()
                .map(|p| {
                    let mut c: PDH_HCOUNTER = std::mem::zeroed();
                    let path = wide(p);
                    (PdhAddEnglishCounterW(query, path.as_ptr(), 0, &mut c) == 0).then_some(c)
                })
                .collect();
            // Rate counters need a first collection before values exist.
            PdhCollectQueryData(query);
            Some(Self { query, counters })
        }
    }

    /// Collects one sample for every counter and reports success. Call once per tick before
    /// [`Query::items`]. PDH handles are internally synchronized, so `&self` is enough.
    pub fn collect_ok(&self) -> bool {
        // SAFETY: query is open.
        unsafe { PdhCollectQueryData(self.query) == 0 }
    }

    /// Every instance's (name, value) for counter `index`, or `None` if unavailable.
    pub fn items(&self, index: usize) -> Option<Vec<(String, f64)>> {
        let counter = (*self.counters.get(index)?)?;
        // SAFETY: the buffer is sized by the first call and outlives the reads.
        unsafe {
            let (mut size, mut count) = (0u32, 0u32);
            let r = PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                std::ptr::null_mut(),
            );
            if r != PDH_MORE_DATA || size == 0 {
                return None;
            }
            let item = std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>();
            let mut buf: Vec<PDH_FMT_COUNTERVALUE_ITEM_W> =
                vec![std::mem::zeroed(); (size as usize).div_ceil(item)];
            if PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut size,
                &mut count,
                buf.as_mut_ptr(),
            ) != 0
            {
                return None;
            }
            Some(
                buf[..count as usize]
                    .iter()
                    .filter(|i| i.FmtValue.CStatus == 0)
                    .map(|i| {
                        let mut len = 0;
                        while *i.szName.add(len) != 0 {
                            len += 1;
                        }
                        let name =
                            String::from_utf16_lossy(std::slice::from_raw_parts(i.szName, len));
                        (name, i.FmtValue.Anonymous.doubleValue)
                    })
                    .collect(),
            )
        }
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        // SAFETY: query was opened by PdhOpenQueryW.
        unsafe {
            PdhCloseQuery(self.query);
        }
    }
}

/// A display adapter found through D3DKMT.
#[derive(Debug, Clone, PartialEq)]
pub struct Adapter {
    /// Matches [`super::parse::luid_key`] of the adapter's PDH instances.
    pub luid_key: String,
    pub luid: (i32, u32),
    pub name: String,
    pub kind: GpuKind,
    pub dedicated_bytes: u64,
}

/// Less dedicated memory than this counts as integrated (an iGPU's small carve-out).
const INTEGRATED_BELOW: u64 = 1024 * 1024 * 1024;

pub fn kind_for(dedicated_bytes: u64) -> GpuKind {
    if dedicated_bytes < INTEGRATED_BELOW {
        GpuKind::Integrated
    } else {
        GpuKind::Discrete
    }
}

/// Hardware adapters, skipping the software "Microsoft Basic Render Driver".
pub fn adapters() -> Vec<Adapter> {
    use windows_sys::Wdk::Graphics::Direct3D::{
        D3DKMT_ADAPTERINFO, D3DKMT_ADAPTERREGISTRYINFO, D3DKMT_CLOSEADAPTER, D3DKMT_ENUMADAPTERS2,
        D3DKMT_QUERYADAPTERINFO, D3DKMT_SEGMENTSIZEINFO, D3DKMTCloseAdapter, D3DKMTEnumAdapters2,
        D3DKMTQueryAdapterInfo, KMTQAITYPE_ADAPTERREGISTRYINFO, KMTQAITYPE_GETSEGMENTSIZE,
    };
    let mut out = Vec::new();
    // SAFETY: the first call sizes the array, the second fills it; every returned adapter handle
    // is closed before returning. Query buffers are correctly sized locals.
    unsafe {
        let mut e = D3DKMT_ENUMADAPTERS2::default();
        if D3DKMTEnumAdapters2(&mut e) != 0 || e.NumAdapters == 0 {
            return out;
        }
        let mut list: Vec<D3DKMT_ADAPTERINFO> = vec![std::mem::zeroed(); e.NumAdapters as usize];
        e.pAdapters = list.as_mut_ptr();
        if D3DKMTEnumAdapters2(&mut e) != 0 {
            return out;
        }
        for a in &list[..e.NumAdapters as usize] {
            let mut reg: D3DKMT_ADAPTERREGISTRYINFO = std::mem::zeroed();
            let mut q = D3DKMT_QUERYADAPTERINFO {
                hAdapter: a.hAdapter,
                Type: KMTQAITYPE_ADAPTERREGISTRYINFO,
                pPrivateDriverData: (&mut reg as *mut D3DKMT_ADAPTERREGISTRYINFO).cast(),
                PrivateDriverDataSize: std::mem::size_of::<D3DKMT_ADAPTERREGISTRYINFO>() as u32,
            };
            let name = if D3DKMTQueryAdapterInfo(&mut q) == 0 {
                let s = &reg.AdapterString;
                let len = s.iter().position(|&c| c == 0).unwrap_or(s.len());
                String::from_utf16_lossy(&s[..len]).trim().to_string()
            } else {
                String::new()
            };
            let mut seg: D3DKMT_SEGMENTSIZEINFO = std::mem::zeroed();
            let mut q = D3DKMT_QUERYADAPTERINFO {
                hAdapter: a.hAdapter,
                Type: KMTQAITYPE_GETSEGMENTSIZE,
                pPrivateDriverData: (&mut seg as *mut D3DKMT_SEGMENTSIZEINFO).cast(),
                PrivateDriverDataSize: std::mem::size_of::<D3DKMT_SEGMENTSIZEINFO>() as u32,
            };
            let dedicated = if D3DKMTQueryAdapterInfo(&mut q) == 0 {
                seg.DedicatedVideoMemorySize
            } else {
                0
            };
            D3DKMTCloseAdapter(&D3DKMT_CLOSEADAPTER {
                hAdapter: a.hAdapter,
            });
            if name.is_empty() || name.contains("Basic Render") {
                continue;
            }
            let luid = (a.AdapterLuid.HighPart, a.AdapterLuid.LowPart);
            out.push(Adapter {
                luid_key: super::parse::luid_key_from_parts(luid.0, luid.1),
                luid,
                name,
                kind: kind_for(dedicated),
                dedicated_bytes: dedicated,
            });
        }
    }
    out
}

/// Driver-reported temperature (°C) of the adapter with `luid`, the value Task Manager shows.
pub fn adapter_temp(luid: (i32, u32)) -> Option<f32> {
    use windows_sys::Wdk::Graphics::Direct3D::{
        D3DKMT_ADAPTER_PERFDATA, D3DKMT_CLOSEADAPTER, D3DKMT_OPENADAPTERFROMLUID,
        D3DKMT_QUERYADAPTERINFO, D3DKMTCloseAdapter, D3DKMTOpenAdapterFromLuid,
        D3DKMTQueryAdapterInfo, KMTQAITYPE_ADAPTERPERFDATA,
    };
    use windows_sys::Win32::Foundation::LUID;
    // SAFETY: the opened adapter handle is closed before returning.
    unsafe {
        let mut open = D3DKMT_OPENADAPTERFROMLUID {
            AdapterLuid: LUID {
                LowPart: luid.1,
                HighPart: luid.0,
            },
            hAdapter: 0,
        };
        if D3DKMTOpenAdapterFromLuid(&mut open) != 0 {
            return None;
        }
        let mut perf: D3DKMT_ADAPTER_PERFDATA = std::mem::zeroed();
        let mut q = D3DKMT_QUERYADAPTERINFO {
            hAdapter: open.hAdapter,
            Type: KMTQAITYPE_ADAPTERPERFDATA,
            pPrivateDriverData: (&mut perf as *mut D3DKMT_ADAPTER_PERFDATA).cast(),
            PrivateDriverDataSize: std::mem::size_of::<D3DKMT_ADAPTER_PERFDATA>() as u32,
        };
        let ok = D3DKMTQueryAdapterInfo(&mut q) == 0;
        D3DKMTCloseAdapter(&D3DKMT_CLOSEADAPTER {
            hAdapter: open.hAdapter,
        });
        // Deci-Celsius; adapters without a sensor report 0.
        (ok && perf.Temperature > 0)
            .then(|| super::parse::plausible_temp(perf.Temperature as f64 / 10.0))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_adapter_kind_by_dedicated_memory() {
        assert_eq!(kind_for(512 * 1024 * 1024), GpuKind::Integrated);
        assert_eq!(kind_for(0), GpuKind::Integrated);
        assert_eq!(kind_for(8 * 1024 * 1024 * 1024), GpuKind::Discrete);
    }

    /// Machine-dependent: `cargo test adapters_live -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn adapters_live() {
        for a in adapters() {
            println!("{a:?} temp={:?}", adapter_temp(a.luid));
        }
    }
}
