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
  type: string
  linked: boolean
}

async function fetchResources({ providerId, userId }: ResourceParams) {
  const urlPath = `/resources/${providerId}/${userId}`
  const { data } = await httpClient.get<Resource[]>(urlPath)

  return data
}

export function useListResources({ providerId, userId }: ResourceParams) {
  return useQuery({
    queryKey: ["resources", providerId, userId],
    queryFn: () => fetchResources({ providerId, userId }),
  })
}
