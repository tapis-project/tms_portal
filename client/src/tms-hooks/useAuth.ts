import { useQuery } from "@tanstack/react-query"
import Cookies from "js-cookie"

export function useAuth() {
  return useQuery({
    queryKey: ["auth-cookie"],
    queryFn: () => Cookies.get("tmstoken") ?? null,
  })
}
