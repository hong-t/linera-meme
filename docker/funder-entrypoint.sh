#!/bin/bash

PATH=/:$PATH python3.10 src/funder.py --swap-application-id "$SWAP_APPLICATION_ID" --wallet-host "$WALLET_HOST" --wallet-owner "$WALLET_OWNER" --wallet-chain "$WALLET_CHAIN" --swap-host "$SWAP_HOST" --proxy-host "$PROXY_HOST" --proxy-application-id "$PROXY_APPLICATION_ID"
