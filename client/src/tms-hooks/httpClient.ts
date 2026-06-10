import axios from "axios"
import Cookies from "js-cookie"

export const httpClient = axios.create({ timeout: 30000 })

httpClient.interceptors.request.use((config) => {
  // placeholder for custom interceptors
  // can also handle token refresh
  // config.headers['x-tms-token'] = 'XXX'
  if (Cookies.get("tmstoken")) {
    config.headers["Authorization"] = `Bearer ${Cookies.get("tmstoken")}`
  }
  return config
})
