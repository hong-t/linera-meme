import { defineStore } from 'pinia'
import { Application, GetApplicationsRequest } from './types'
import { ApolloClient } from '@apollo/client/core'
import { getClientOptions } from 'src/apollo'
import { provideApolloClient, useQuery } from '@vue/apollo-composable'
import { APPLICATIONS } from 'src/graphql'
import { graphqlResult } from 'src/utils'
import { BlobGateway } from '../blob'
import { Meme } from '../meme'

const options = /* await */ getClientOptions()
const apolloClient = new ApolloClient(options)

export const useAmsStore = defineStore('ams', {
  state: () => ({
    applications: [] as Array<Application>
  }),
  actions: {
    getApplications(
      req: GetApplicationsRequest,
      done?: (error: boolean, rows?: Application[]) => void
    ) {
      const { /* result, refetch, fetchMore, */ onResult, onError } =
        provideApolloClient(apolloClient)(() =>
          useQuery(
            APPLICATIONS,
            {
              createdAfter: req.createdAfter,
              limit: req.limit,
              endpoint: 'ams'
            },
            {
              fetchPolicy: 'network-only'
            }
          )
        )

      onResult((res) => {
        const applications = graphqlResult.data(
          res,
          'applications'
        ) as Application[]
        this.appendApplications(applications)
        done?.(false, applications)
      })

      onError(() => {
        done?.(true)
      })
    },
    appendApplications(applications: Application[]) {
      applications.forEach((application) => {
        const index = this.applications.findIndex(
          (el) => el.applicationId === application.applicationId
        )
        this.applications.splice(
          index >= 0 ? index : 0,
          index >= 0 ? 1 : 0,
          application
        )
      })
    }
  },
  getters: {
    applicationLogo(): (application: Application) => string {
      return (application: Application) => {
        return BlobGateway.imagePath(
          application?.logoStoreType,
          application?.logo
        )
      }
    },
    existMeme(): (name?: string, ticker?: string) => boolean {
      return (name?: string, ticker?: string) => {
        return (
          this.applications.findIndex((el) => {
            let ok = true
            const meme = JSON.parse(el.spec) as Meme
            if (name?.length) {
              ok = ok && meme.name === name
            }
            if (ticker?.length) {
              ok = ok && meme.ticker === ticker
            }
            return ok
          }) >= 0
        )
      }
    },
    application(): (applicationId: string) => Application | undefined {
      return (applicationId: string) => {
        return this.applications.find(
          (el) => el.applicationId === applicationId
        )
      }
    }
  }
})
