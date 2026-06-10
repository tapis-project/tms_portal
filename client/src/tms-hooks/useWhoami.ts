import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type Whoami = {
  name: string
  user_name: string
  idpDisplayName: string
  organization: string
}

const fetchWhoami = async () => {
  const { data } = await httpClient.get<{ result: Whoami }>("/login/whoami")
  return data?.result
}

export const useWhoami = ({ enabled }: { enabled?: boolean }) => {
  return useQuery({
    queryKey: ["auth", "whoami"],
    queryFn: () => fetchWhoami(),
    enabled: enabled ?? true,
  })
}
