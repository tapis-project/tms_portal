import {
  Card,
  CardHeader,
  CardTitle,
  //CardDescription,
  CardContent,
  CardAction,
} from "@/components/ui/card"
import { Button } from "@/components/ui/button"

import { Building2, Unplug } from "lucide-react"
import type { Provider } from "@/tms-hooks"

export function ProviderCard({
  provider,
  children,
}: React.PropsWithChildren<{ provider: Provider; identity?: string }>) {
  return (
    <Card className="border-border/60 shadow-sm">
      <CardHeader>
        {/* {identity && (
          <CardTitle className="flex min-w-0 gap-2">
            <UserRound />
            <span className="break-all">{identity}</span>
          </CardTitle>
        )}
          */}
        <CardTitle className="min-w-0 gap-2">
          <p className="flex items-center gap-1">
            <Building2 className="mr-1 inline" />
            <span>{provider.name}</span>
          </p>
          <p>{provider.description}</p>
        </CardTitle>

        <CardAction>
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
