import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type Provider = {
  id: string
  name: string
  clientId: string
  oauth2TokenUrl: string

  userInfoUrl?: string
  institution?: string
  location?: string
  description?: string
}

const fetchProviders = async () => {
  const { data } = await httpClient.get<{ result: Provider[] }>(
    "/resources/providers"
  )
  return data?.result
}

export const useListProviders = () => {
  return useQuery({
    queryKey: ["providers"],
    queryFn: () => fetchProviders(),
  })
}
