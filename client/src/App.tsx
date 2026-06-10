import { Plus } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

import {
  useListProviderLinks,
  useListProviders,
  useListResources,
} from "./tms-hooks"
import { ResourceCard } from "./components/tms-ui/ResourceCard"
import { ProviderCard } from "./components/tms-ui/ProviderCard"
import { UserMenu } from "./components/tms-ui/UserMenu"
import { useAuth } from "./tms-hooks/useAuth"
import { Separator } from "./components/ui/separator"

function ResourceCardGrid({
  userId,
  providerId,
}: {
  userId: string
  providerId: string
}) {
  const { data: ResourceList } = useListResources({ providerId, userId })
  if (!ResourceList) return null
  return (
    <>
      {/* Grids items will stack vertically in order to  */}
      <div className="grid gap-2 sm:grid-cols-[repeat(auto-fit,minmax(400px,1fr))]">
        {ResourceList.map((resource) => (
          <ResourceCard resource={resource} key={resource.id} />
        ))}
      </div>
    </>
  )
}

function LinkIdentityModal() {
  const { data: providerList } = useListProviders()
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="sm" variant="outline">
          <Plus /> Add Identity
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Link a New Provider Identity</DialogTitle>
        </DialogHeader>
        <Separator />
        {providerList?.map((provider) => {
          return (
            <div
              key={provider.id}
              className="flex items-center justify-between gap-4"
            >
              <span>
                {provider.name} ({provider.id})
              </span>
              <Button size="sm" className="bg-(--success)">
                Connect
              </Button>
            </div>
          )
        })}
        <Separator />
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function ProviderCardList() {
  const { data: providerList } = useListProviders()
  const { data: providerLinks } = useListProviderLinks()

  if (!providerList || !providerLinks) return null
  const providersWithIdentities = providerList.map((p) => ({
    ...p,
    linkedIdentities: providerLinks
      .filter((link) => link.providerId === p.id)
      .map((link) => link.providerIdentity),
  }))
  return providersWithIdentities
    .filter((p) => p.linkedIdentities.length > 0)
    .map((provider) =>
      provider.linkedIdentities.map((identity) => (
        <ProviderCard
          key={`${provider.id}_${identity}`}
          provider={provider}
          identity={identity}
        >
          <ResourceCardGrid userId={identity} providerId={provider.id} />
        </ProviderCard>
      ))
    )
}

function App() {
  const { data: isAuthenticated } = useAuth()
  console.log(isAuthenticated)
  return (
    <div className="min-h-screen bg-muted/30">
      <header className="border-b bg-background/95 backdrop-blur supports-backdrop-filter:bg-background/60">
        <div className="mx-auto flex max-w-5xl flex-row items-center justify-between gap-4 px-4 py-4 sm:px-6 lg:px-8">
          <h1 className="text-2xl font-semibold tracking-tight">
            Trust Manager System
          </h1>

          <UserMenu />
        </div>
      </header>

      <main className="mx-auto max-w-5xl space-y-2 px-4 py-6 sm:px-6 lg:px-8">
        {!isAuthenticated && (
          <div>
            Please{" "}
            <a
              href="/login?idp_id=globus_idp&redirect_uri=https://tms-auth-service.tacc.cloud/"
              className="text-primary underline-offset-4 hover:underline"
            >
              log in
            </a>{" "}
            to manage resources.
          </div>
        )}
        {!!isAuthenticated && (
          <>
            <h2 className="flex items-center gap-2 text-lg font-semibold">
              Linked Identities <LinkIdentityModal />
            </h2>
            <ProviderCardList></ProviderCardList>
          </>
        )}
      </main>
    </div>
  )
}

export default App
