# Omniscience for Stream Deck

A Stream Deck plugin with one action, **Omniscience reading**. It shows one reading from
Omniscience as the key's title and refreshes every 2 seconds. If the reading cannot be fetched,
the key shows `--`.

**This plugin has not been tested on a real Stream Deck yet.** It is plain HTML and JavaScript
with no build step. Expect rough edges and please report problems.

## Install

1. Copy the `io.github.joezhuo2.omniscience.sdPlugin` folder into Stream Deck's Plugins folder:
   - Windows: `%APPDATA%\Elgato\StreamDeck\Plugins`
   - macOS: `~/Library/Application Support/com.elgato.StreamDeck/Plugins`
2. Restart Stream Deck.
3. The **Omniscience** category now appears in the action list.

## Set up

1. In Omniscience, open Settings > Integrations and turn on **Local API**.
2. Copy the token. It is the part after `token=` in the address shown in the settings.
3. In Stream Deck, drag **Omniscience reading** onto a key.
4. Choose the reading, enter the API port (default 9124) and paste the token.

The token is required. The plugin is a web page, and the API asks web pages for the token.

## Readings

CPU %, CPU temperature, memory %, integrated GPU %, discrete GPU %, disk %, download speed,
upload speed and power. A reading the PC does not have shows `--`.

Settings are saved per key.
