import { gql } from '@apollo/client/core'

export const BALANCES = gql`
  query balances($chainOwners: JSONObject!) {
    balances(chainOwners: $chainOwners)
  }
`
