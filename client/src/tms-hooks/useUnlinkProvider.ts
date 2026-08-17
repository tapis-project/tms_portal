import { useMutation, useQueryClient } from "@tanstack/react-query"
import { httpClient } from "./httpClient"

const deleteProviderLink = async ({ id }: { id: number }) => {
  await httpClient.delete(`/resources/providers/links/${id}`)
}

export const useUnlinkProvider = ({ id }: { id: number }) => {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: () => deleteProviderLink({ id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["providerLinks"] })
    },
  })
}
