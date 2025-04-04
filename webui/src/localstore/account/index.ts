import { Account } from './types'

export class _Account {
  static applicationUrl = (
    host: string,
    endpoint: string,
    application: Account
  ) => {
    if (!application.owner) return
    const chainId = application.chainId
    const applicationId = application.owner.split(':')[1]
    return `http://${host}/api/${endpoint}/chains/${chainId}/applications/${applicationId}`
  }

  static accountDescription = (account: Account) => {
    let description = account.chainId
    if (account.owner) description += ':' + account.owner
    return description
  }

  static accountOwner = (account: Account) => {
    if (!account.owner) return
    return account.owner.split(':')[1]
  }
}

export * from './types'
