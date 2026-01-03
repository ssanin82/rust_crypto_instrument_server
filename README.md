# Instrument Server Example

## Purpose
- Define a clear trading symbology for handling spot and perp trading in crypto.
- Define the structure and automatically collect reference data from selected exchanges.
- Store the reference data in the local DB

## Problem it Solves
In trading systems I’ve dealt with in the past, there was often a need to define a clear internal symbology and get instrument reference data efficiently. For example, spots and perps shared the same symbol internally, which created confusion for strategies that work with both simultaneously and made it very difficult to work with exchanges that allow access to both perps and spots on the same account using the same protocol.

By 'reference data,' I mean the data necessary to form valid orders (e.g., lot size, tick size, minimum notional, maximum limit order size, etc.), but which is usually not streamed from the exchange. Thus, in many startups, people just start defining this reference data manually in configuration files, often multiple times, specific for each strategy. As a result, every time the exchange updates an instrument’s reference data, all of those configuration files need to be detected, found, and manually updated. A human error while doing this may cause the trade to stop or result in a loss of money.

## Tech Stack
- Rust
- SQLite

## Prerequisites

- SQLite: ```sudo apt update; sudo apt install -y sqlite3```
