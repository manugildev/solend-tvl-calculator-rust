# Solend Total Value Locked Calculator Demo 

Rust rewritten utility to compute total deposits and borrows from on-chain data. Works by fetching obligation for every user and computing a sum of `deposits` and `borrows` (obligation properties) for each asset.

Useful as a demo for reading on-chain Solend data.

## Run Demo

    cargo run

### Example output:

    Number of users: 92097
    Total deposits:
    SOL: 275061194411551
    FTT: 38254584941
    BTC: 442056269
    USDC: 22889058124156
    SRM: 149691930987
    ETH: 7057942032
    USDT: 2768283665774
    RAY: 325031428873
    Total borrows:
    SOL: 49594719385258
    USDC: 15791074678867
    FTT: 3800511105
    USDT: 1508131948858
    RAY: 256277952020
    SRM: 41497749279
    ETH: 3718970301
    BTC: 81119486
