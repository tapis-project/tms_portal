import { CheckIcon, Server, Link2, Unplug } from "lucide-react"

import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardAction,
} from "@/components/ui/card"

import { Button } from "@/components/ui/button"

import { Badge } from "@/components/ui/badge"
import type { Resource } from "@/tms-hooks"

export function ResourceCard({ resource }: { resource: Resource }) {
  return (
    <Card
      key={resource.name}
      className="rounded-lg border bg-background shadow-md"
    >
      <CardHeader>
        <CardTitle className="text-xl">
          <span className="inline-block align-middle break-all">
            {resource.name}
          </span>{" "}
          {resource.linked ? (
            <Badge
              className="rounded-[10px] bg-(--success) align-middle"
              variant="default"
            >
              <CheckIcon />
              Linked
            </Badge>
          ) : (
            <Badge className="rounded-[10px] align-middle" variant="secondary">
              Unlinked
            </Badge>
          )}
        </CardTitle>
        <CardDescription className="col-span-2 space-y-2">
          <p>
            <Server className="inline size-5" /> {resource.type}
          </p>
        </CardDescription>

        <CardAction className="col-span-2 col-start-1 sm:col-span-1 sm:col-start-2">
          {resource.linked ? (
            <Button variant="destructive">
              <Unplug className="mr-1 size-4" />
              Unlink
            </Button>
          ) : (
            <Button variant="outline">
              <Link2 className="mr-1 size-4" />
              Link
            </Button>
          )}
        </CardAction>
      </CardHeader>
      <CardContent>
        <CardDescription>
          <p>{resource.description}</p>
        </CardDescription>
      </CardContent>
    </Card>
  )
}
