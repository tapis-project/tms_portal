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

const fetchProviders = async ({
  linkedOnly,
}: { linkedOnly?: boolean } = {}) => {
  const { data } = await httpClient.get<{ result: Provider[] }>(
    "/resources/providers",
    { params: { linked_only: linkedOnly } }
  )
  return data?.result
}

export const useListProviders = ({
  linkedOnly,
}: { linkedOnly?: boolean } = {}) => {
  return useQuery({
    queryKey: ["providers", linkedOnly],
    queryFn: () => fetchProviders({ linkedOnly }),
  })
}
