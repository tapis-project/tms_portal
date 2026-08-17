import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type ProviderLink = {
  id: number
  tms_identity: string
  resource_provider_account: string
  resource_provider_uuid: string
  resource_provider_id: string
  resource_provider_name: string
  last_login: string
  enabled: false
}

const fetchProviderLinks = async () => {
  const { data } = await httpClient.get<{ result: ProviderLink[] }>(
    "/resources/providers/links"
  )
  return data?.result
}

export const useListProviderLinks = () => {
  return useQuery({
    queryKey: ["providerLinks"],
    queryFn: () => fetchProviderLinks(),
  })
}
