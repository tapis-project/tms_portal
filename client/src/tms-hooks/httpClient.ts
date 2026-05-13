import axios from "axios"

export const httpClient = axios.create({ timeout: 30000 })

httpClient.interceptors.request.use((config) => {
  // placeholder for custom interceptors
  // can also handle token refresh
  // config.headers['x-tms-token'] = 'XXX'
  return config
})
