import { Plus } from "lucide-react"
import { Button } from "@/components/ui/button"

import { useListProviders, useListResources } from "./tms-hooks"
import { ResourceCard } from "./components/tms-ui/ResourceCard"
import { ProviderCard } from "./components/tms-ui/ProviderCard"
import { UserMenu } from "./components/tms-ui/UserMenu"
import { useAuth } from "./tms-hooks/useAuth"

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

function ProviderCardList() {
  const { data: providerList } = useListProviders()
  if (!providerList) return null
  return providerList
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
              Linked Identities{" "}
              <Button size="sm" variant="outline">
                <Plus /> Add Identity
              </Button>
            </h2>
            <ProviderCardList></ProviderCardList>
          </>
        )}
      </main>
    </div>
  )
}

export default App
