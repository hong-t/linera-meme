import requests


class Wallet:
    def __init__(self, wallet_host, owner, chain):
        self.wallet_host = wallet_host
        self.wallet_url = f'http://{wallet_host}'
        self.owner = owner
        self.chain = chain

    def account(self):
        return f'{self.chain}:{self.owner}'

    def _chain(self):
        return self.chain

    def _wallet_url(self):
        return self.wallet_url

    def balance(self):
        chain_owners = f'''[{{
            chainId: "{self.chain}",
            owners: ["{self.owner}"]
        }}]'''
        json = {
            'query': f'query {{\n balances(chainOwners:{chain_owners}) \n}}'
        }
        try:
            resp = requests.post(self.wallet_url, json=json)
        except:
            return 0

        if 'data' not in resp.json():
            return 0

        balances = resp.json()['data']['balances']
        if self.chain not in balances:
            print(f'{self.chain} not in wallet {self.wallet_url}: {resp.text}')
            return 0

        balances = balances[self.chain]
        chain_balance = float(balances['chainBalance']) if 'chainBalance' in balances else 0

        balances = balances['ownerBalances']
        owner_balance = float(balances[self.owner]) if self.owner in balances else 0

        return chain_balance + owner_balance

