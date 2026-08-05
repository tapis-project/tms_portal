import React from "react"
import { Plus, Server } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"

import { useListProviders, useListResources } from "./tms-hooks"
//import { ResourceCard } from "./components/tms-ui/ResourceCard"
import { ProviderCard } from "./components/tms-ui/ProviderCard"
import { UserMenu } from "./components/tms-ui/UserMenu"
import { useAuth } from "./tms-hooks/useAuth"
import { Separator } from "./components/ui/separator"
//import { ProviderWizard } from "./components/tms-ui/ProviderWizard"

/*
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
      <div className="grid gap-2 sm:grid-cols-[repeat(auto-fit,minmax(400px,1fr))]">
        {ResourceList.map((resource) => (
          <ResourceCard resource={resource} key={resource.id} />
        ))}
      </div>
    </>
  )
}
*/

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
              <Button size="sm" asChild>
                <a
                  href={`/resources/providers/authorize?provider_id=${provider.id}&redirect_url=${window.location.origin}`}
                >
                  Connect
                </a>
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
  const { data: providerList } = useListProviders({ linkedOnly: true })

  if (!providerList) return null

  return providerList.map((provider) => (
    <ProviderCard key={`${provider.id}`} provider={provider}>
      {/* <ResourceCardGrid userId={"ok"} providerId={provider.id} />  */}
      <ResourceCardSelector providerId={provider.id} />
    </ProviderCard>
  ))
}

import { Checkbox } from "@/components/ui/checkbox"
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@/components/ui/field"

function ResourceCardSelector({ providerId }: { providerId: string }) {
  const { data: resourceList } = useListResources({ providerId, userId: "ok" })
  const [selectedResources, setSelectedResources] = React.useState<Set<string>>(
    new Set()
  )
  React.useEffect(
    () => setSelectedResources(new Set(resourceList?.map((r) => r.id))),
    [resourceList]
  )

  if (!resourceList) return null

  function handleSelect(id: string, checked: boolean) {
    const newSelectedResources = new Set(selectedResources)
    if (!checked) {
      newSelectedResources.delete(id)
    } else {
      newSelectedResources.add(id)
    }
    setSelectedResources(newSelectedResources)
  }

  return (
    <div className="flex flex-col items-center gap-4">
      <FieldGroup className="grid gap-2 sm:grid-cols-[repeat(auto-fit,minmax(400px,1fr))]">
        {resourceList.map((resource) => (
          <FieldLabel key={resource.id}>
            <Field
              orientation="horizontal"
              onChange={() => console.log("change fired")}
            >
              <Checkbox
                id="toggle-checkbox-2"
                name="toggle-checkbox-2"
                checked={selectedResources.has(resource.id)}
                onCheckedChange={(e: boolean) => handleSelect(resource.id, e)}
              />
              <FieldContent>
                <FieldTitle className="text-xl">
                  <Server className="mr-1 inline size-5" />
                  <span className="inline-block align-middle break-all">
                    {resource.name}
                  </span>{" "}
                </FieldTitle>
                <FieldDescription>{resource.description}</FieldDescription>
              </FieldContent>
            </Field>
          </FieldLabel>
        ))}
      </FieldGroup>
      <Button asChild>
        <a href="#">Confirm Delegation and Return to Science Gateway</a>
      </Button>
    </div>
  )
}

function App() {
  const { data: isAuthenticated } = useAuth()
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
            {/* <ProviderWizard /> */}
          </>
        )}
      </main>
    </div>
  )
}

export default App
