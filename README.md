# Marci
Data exposer for https://github.com/code-monad/ckb-analyzer

## Quick start:

`cargo run -- --db-url postgre://127.0.0.1/ckb --bind 0.0.0.0:1800`

and visit `127.0.0.1:1800/peer`

## How-to bundle frontend page

Checkout [Marci-Rebound](https://github.com/code-monad/marci-rebound), build and copy the `dist` of `Marci-Rebound` output into [/dist](./dist)

```bash
git clone https://github.com/code-monad/marci-rebound && cd marci-rebound
ng build --configuration=production
cp dist/marci-rebound/* ${MARCI_SRC_DIR}/dist # replace with your work dir
```
