<template>
  <q-btn flat rounded @click='onConnectClick'>
    <q-menu v-if='publicKey?.length' anchor='bottom right' self='top right' :offset='[0, 8]'>
      <q-card flat>
        <div class='row flex justify-center items-center' :style='{margin: "36px 0 36px 0", fontSize: "28px"}'>
          <q-space />
          <div :style='{marginLeft: "8px"}'>
            {{ (Number(accountBalance) + Number(chainBalance)).toFixed(4) }}
          </div>
          <div :style='{margin: "8px 0 0 8px", fontSize: "12px"}'>
            TLINERA
          </div>
          <q-space />
        </div>
        <q-separator :style='{margin: "0 0 16px 0"}' />
        <div class='popup-padding'>
          <div class='row'>
            <div :style='{width: "24px"}'>
              <q-img :src='addressIcon' width='16px' height='16px' />
            </div>
            <div :style='{width: "calc(100% - 24px)"}'>
              <div class='text-grey-6'>
                Address
              </div>
              <div class='row'>
                <div class='text-bold'>
                  {{ shortid.shortId(publicKey, 14) }}
                </div>
                <q-space />
                <div :style='{marginLeft: "8px"}' class='cursor-pointer'>
                  <q-img :src='copyIcon' width='16px' height='16px' @click.stop='(evt) => _copyToClipboard(publicKey, evt)' />
                </div>
              </div>
              <div class='text-grey-6'>
                {{ Number(accountBalance).toFixed(4) }}
              </div>
            </div>
          </div>
          <div class='row' :style='{margin: "12px 0 0 0"}'>
            <div :style='{width: "24px"}'>
              <q-img :src='microchainIcon' width='16px' height='16px' />
            </div>
            <div :style='{width: "calc(100% - 24px)"}'>
              <div class='text-grey-6'>
                Microchain
              </div>
              <div class='row'>
                <div class='text-bold'>
                  {{ shortid.shortId(chainId, 14) }}
                </div>
                <q-space />
                <div :style='{marginLeft: "8px"}' class='cursor-pointer'>
                  <q-img :src='copyIcon' width='16px' height='16px' @click='(evt) => _copyToClipboard(chainId, evt)' />
                </div>
              </div>
              <div class='text-grey-6'>
                {{ Number(chainBalance).toFixed(4) }}
              </div>
            </div>
          </div>
          <q-btn
            flat rounded class='bg-red-6 full-width text-white'
            @click='onLogoutClick'
            label='Logout'
            :style='{margin: "8px 0 0 0"}'
          />
          <div class='text-grey-6 text-center' :style='{margin: "8px 0 4px 0", fontSize: "12px"}'>
            Powered by CheCko
          </div>
        </div>
      </q-card>
    </q-menu>
    <q-img src='https://avatars.githubusercontent.com/u/107513858?s=48&v=4' width='24px' height='24px' />
    <div :style='{margin: "2px 0 0 8px"}' class='text-grey-9 text-bold'>
      {{ publicKey?.length ? shortid.shortId(publicKey, 6) : 'Connect Wallet' }}
      <span v-if='publicKey?.length'>
        <span class='text-grey-4'>|</span> {{ (Number(accountBalance) + Number(chainBalance)).toFixed(4) }}
      </span>
    </div>
  </q-btn>
</template>
<script setup lang='ts'>
import { computed, onMounted, watch } from 'vue'
import { Cookies } from 'quasar'
import { user, block, account } from 'src/localstore'
import { shortid } from 'src/utils'
import { Web3 } from 'web3'
import { addressIcon, microchainIcon, copyIcon } from 'src/assets'
import { BALANCES } from 'src/graphql'
import { dbModel, rpcModel } from 'src/model'
import { _copyToClipboard } from 'src/utils/copy_to_clipboard'

const _user = user.useUserStore()
const _block = block.useBlockStore()

const publicKey = computed(() => _user.publicKey?.trim())
const chainId = computed(() => _user.chainId?.trim())
const accountBalance = computed(() => _user.accountBalance)
const chainBalance = computed(() => _user.chainBalance)

const blockHeight = computed(() => _block.blockHeight)

const getProviderState = () => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  window.linera.request({
    method: 'metamask_getProviderState'
  }).then(async (result) => {
    _user.chainId = ((result as Record<string, string>).chainId).substring(2)
    _user.publicKey = ((result as Record<string, string>).accounts)[0]
    Cookies.set('CheCko-Login-Account', _user.publicKey)
    Cookies.set('CheCko-Login-Microchain', _user.chainId)
    await getBalances()
  }).catch((e) => {
    console.log('metamask_getProviderState', e)
  })
}

const onConnectClick = async () => {
  if (!window.linera) {
    return window.open('https://github.com/respeer-ai/linera-wallet.git')
  }

  try {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    const web3 = new Web3(window.linera)
    await web3.eth.requestAccounts()
  } catch (e) {
    // DO NOTHING
  }

  getProviderState()
  await getBalances()
}

const onLogoutClick = () => {
  Cookies.remove('CheCko-Login-Account')
  Cookies.remove('CheCko-Login-Microchain')
  _user.$reset()
}

const walletReadyCall = (f: () => void) => {
  if (!window.linera) {
    return setTimeout(() => walletReadyCall(f), 1000)
  }
  f()
}

onMounted(() => {
  walletReadyCall(() => {
    void getProviderState()
  })
})

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const getBalances = async () => {
  if (!publicKey.value) return
  const owner = await dbModel.ownerFromPublicKey(publicKey.value)
  // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
  window.linera.request({
    method: 'linera_graphqlQuery',
    params: {
      publicKey: publicKey.value,
      query: {
        query: BALANCES.loc?.source?.body,
        variables: {
          chainOwners: [{
            chainId: chainId.value,
            owners: [account._Account.formalizeOwner(owner)]
          }],
          chainId: chainId.value,
          publicKey: publicKey.value
        }
      }
    }
  }).then((result) => {
    const balances = result as rpcModel.Balances
    _user.chainBalance = rpcModel.chainBalance(balances, chainId.value)
    _user.accountBalance = rpcModel.ownerBalance(balances, chainId.value, account._Account.formalizeOwner(owner))
  }).catch((e) => {
    console.log(e)
  })
}

watch(blockHeight, async () => {
  await getBalances()
})

watch(publicKey, async () => {
  await getBalances()
})

</script>

<style lang='sass' scoped>
</style>
