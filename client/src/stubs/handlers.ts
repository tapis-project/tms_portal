import { http, HttpResponse, type JsonBodyType } from "msw"

const providerStubs = [
  {
    id: "tacc",
    name: "Texas Advanced Computing Center (TACC)",
    institution: "University of Texas at Austin",
    location: "Austin, TX",
    description:
      "TACC provides large-scale computing resources for open science research.",
    linkedIdentities: ["jarosenb", "jarosenb_TEST"],
  },
  {
    id: "rp_sdsc",
    name: "San Diego Supercomputer Center (SDSC)",
    institution: "UC San Diego",
    description: "SDSC operates HPC systems designed for science gateways.",
    location: "San Diego, CA, USA",
    linkedIdentities: ["jrosenberg"],
  },
]

const resourceStubs: Record<string, Record<string, JsonBodyType>> = {
  tacc: {
    jarosenb: [
      {
        id: "frontera",
        name: "Frontera",
        type: "compute",
        linked: true,
        description:
          "Deployed in June 2019, Frontera is the fastest supercomputer on a university campus in the U.S.",
      },
      {
        id: "stampede3",
        name: "Stampede3",
        type: "compute",
        linked: false,
        description:
          "Stampede3 is the newest strategic resource for the nation's open science community since entering full production in 2024.",
      },
    ],
    jarosenb_TEST: [
      {
        id: "frontera",
        name: "Frontera",
        type: "compute",
        linked: true,
        description:
          "Deployed in June 2019, Frontera is the fastest supercomputer on a university campus in the U.S.",
      },
      {
        id: "corral",
        name: "Corral",
        type: "storage",
        linked: false,
        description:
          "Corral is TACC's primary data-management and storage resource for supporting the creation and curation of research data collections.",
      },
    ],
  },
  rp_sdsc: {
    jrosenberg: [
      {
        id: "expanse",
        name: "Expanse",
        type: "compute",
        linked: true,
        description:
          "Expanse supports SDSC's vision of “Computing without Boundaries” by increasing the capacity and performance for thousands of users of batch-oriented and science gateway computing.",
      },
      {
        id: "voyager",
        name: "Voyager",
        type: "compute",
        linked: false,
        description:
          "Voyager is an innovative AI system designed specifically for science and engineering research at scale.",
      },
    ],
  },
}

export const handlers = [
  http.get("/providers", () => {
    return HttpResponse.json(providerStubs)
  }),

  http.get<{ provider: string; userId: string }>(
    "/resources/:provider/:userId",
    ({ params }) => {
      return HttpResponse.json(resourceStubs[params.provider][params.userId])
    }
  ),
]
