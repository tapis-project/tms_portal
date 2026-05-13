import { useQuery } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

export type Provider = {
  id: string;
  name: string;
  institution: string;
  location: string;
  description: string;
  linkedIdentities: string[];
}

const fetchProviders = async () => {
  const { data } = await httpClient.get<Provider[]>("/providers")
  return data
}

export const useListProviders = () => {
  return useQuery({
    queryKey: ["providers"],
    queryFn: () => fetchProviders(),
  })
}
