"use client"

import * as React from "react"

import { Card, CardContent, CardFooter } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  type CarouselApi,
} from "@/components/ui/carousel"

import { Checkbox } from "@/components/ui/checkbox"
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldTitle,
} from "@/components/ui/field"

import { useListProviders, useListResources } from "@/tms-hooks"

import { ArrowBigLeft, ArrowBigRight } from "lucide-react"

function LinkIdentityStep() {
  const { data: linkedProviderList } = useListProviders({ linkedOnly: true })
  const { data: providerList } = useListProviders()
  return (
    <>
      {linkedProviderList?.length === 1 && (
        <p>
          It looks like you don't have any resources set up. This wizard will
          help you walk through that process. Select the institutions that you
          are affiliated with.
        </p>
      )}

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
    </>
  )
}

function SelectResourcesStep({ providerId }: { providerId: string }) {
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
    <div className="flex flex-col gap-4">
      <p className="font-bold">
        Tapis via TMS is requesting full access to the following TACC resources
        on your behalf:
      </p>
      <FieldGroup className="max-w-sm gap-4">
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
                <FieldTitle>{resource.name}</FieldTitle>
                <FieldDescription>{resource.description}</FieldDescription>
              </FieldContent>
            </Field>
          </FieldLabel>
        ))}
      </FieldGroup>
    </div>
  )
}

export function ProviderWizard() {
  const [api, setApi] = React.useState<CarouselApi>()
  const [active] = React.useState(true)
  const [current, setCurrent] = React.useState(0)
  const [count, setCount] = React.useState(0)

  React.useEffect(() => {
    if (!api) {
      return
    }

    setCount(api.scrollSnapList().length)
    setCurrent(api.selectedScrollSnap() + 1)

    api.on("select", () => {
      setCurrent(api.selectedScrollSnap() + 1)
    })
  }, [api])

  const contentSteps = [
    <LinkIdentityStep />,
    <SelectResourcesStep providerId="tacc" />,
    <div className="flex flex-1 flex-col items-center justify-center gap-2">
      <p>Delegation Complete</p>
      <p>
        <Button>Return to Science Gateway</Button>
      </p>
    </div>,
  ]

  return (
    <div className="mx-auto max-w-[10rem] sm:max-w-lg">
      <Carousel setApi={setApi} className="w-full max-w-lg" opts={{ active }}>
        <CarouselContent>
          {Array.from({ length: 3 }).map((_, index) => (
            <CarouselItem key={index}>
              <Card className="m-px">
                <CardContent className="flex h-87.5 flex-col items-center justify-between p-6">
                  {}
                  <></>
                  {contentSteps[index]}
                  <p></p>
                </CardContent>
                <CardFooter className="justify-between">
                  {" "}
                  <Button
                    className="w-25"
                    variant="outline"
                    onClick={() => {
                      api?.scrollTo(current - 2)
                    }}
                  >
                    <ArrowBigLeft />
                    Previous
                  </Button>
                  <Button
                    className="w-25"
                    variant="outline"
                    onClick={() => {
                      api?.scrollTo(current)
                    }}
                  >
                    Next <ArrowBigRight />
                  </Button>
                </CardFooter>
              </Card>
            </CarouselItem>
          ))}
        </CarouselContent>
      </Carousel>
      <div className="py-2 text-center text-sm text-muted-foreground">
        Step {current} of {count}
      </div>
    </div>
  )
}
