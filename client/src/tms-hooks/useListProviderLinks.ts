import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type ProviderLink = {
  providerId: string
  tmsIdentity: string
  providerIdentity: string
}

const fetchProviderLinks = async () => {
  const { data } = await httpClient.get<{ result: ProviderLink[] }>(
    "/resource/provider-links"
  )
  return data?.result
}

export function useListProviderLinks() {
  return useQuery({
    queryKey: ["providers", "links"],
    queryFn: () => fetchProviderLinks(),
  })
}
