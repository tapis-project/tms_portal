import { Server } from "lucide-react"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  // CardAction,
} from "@/components/ui/card"

//import { Button } from "@/components/ui/button"

//import { Badge } from "@/components/ui/badge"
import type { Resource } from "@/tms-hooks"

export function ResourceCardField({ resource }: { resource: Resource }) {
  return (
    <Card
      key={resource.name}
      className="rounded-lg border bg-background shadow-md"
    >
      <CardHeader>
        <CardTitle className="text-xl">
          <Server className="mr-1 inline size-5" />
          <span className="inline-block align-middle break-all">
            {resource.name}
          </span>{" "}
        </CardTitle>

      </CardHeader>
      <CardContent>
        <CardDescription>
          <p>{resource.description}</p>
        </CardDescription>
      </CardContent>
    </Card>
  )
}
