import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardAction,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"

import { Building2, Unplug, UserRound } from "lucide-react"
import type { Provider } from "@/tms-hooks"

export function ProviderCard({
  provider,
  identity,
  children,
}: React.PropsWithChildren<{ provider: Provider; identity: string }>) {
  return (
    <Card className="border-border/60 shadow-sm">
      <CardHeader>
        <CardTitle className="flex gap-2 min-w-0">
          <UserRound />
          <span className="break-all">{identity}</span>
        </CardTitle>
        <CardDescription className="col-span-2 col-start-1 space-y-1">
          <p>
            <Building2 className="mr-1 inline align-bottom" />
            <span>{provider.name}</span>
          </p>
          <p>{provider.description}</p>
        </CardDescription>

        <CardAction className="col-span-2 col-start-1 sm:col-span-1 sm:col-start-2">
          <Button variant="destructive">
            <Unplug className="mr-2 size-4" />
            Disconnect
          </Button>
        </CardAction>
      </CardHeader>

      <CardContent className="space-y-4">
        <div>
          <h2 className="text-base font-semibold">Available Resources</h2>
          <p className="text-sm text-muted-foreground">
            Link or unlink resources associated with this facility.
          </p>
        </div>

        {children}
      </CardContent>
    </Card>
  )
}
