import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

type ResourceParams = {
  providerId: string
  userId: string
}

export type Resource = {
  id: string
  name: string
  description: string
  provider_id: string
  provider_name: string
}

async function fetchResources({ providerId, userId }: ResourceParams) {
  const urlPath = `/resources/${providerId}/${userId}`
  const { data } = await httpClient.get<{ result: Resource[] }>(urlPath)

  return data?.result
}

export function useListResources({ providerId, userId }: ResourceParams) {
  return useQuery({
    queryKey: ["resources", providerId],
    queryFn: () => fetchResources({ providerId, userId }),
  })
}
