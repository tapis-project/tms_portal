import { http, HttpResponse, type JsonBodyType } from "msw"

const providerStubs = [
  {
    id: "tacc",
    name: "TACC Resource Provider",
    clientId: "tms_dev_client_id",
    oauth2TokenUrl: "https://tacc.tapis.io/v3/oauth2/tokens",
    userInfoUrl: "",
  },
]

const whoamiStub = {
  name: "Jake Rosenberg",
  username:
    "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@f2f321a2-b33a-451f-84a3-9f6d212cf902",
  idpDisplayName: "University of Texas at Austin",
  organization: "University of Texas at Austin",
}
const providerLinkageStubs = [
  {
    providerId: "tacc",
    tmsIdentity: "jarosenb",
    providerIdentity: "jarosenb",
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
  http.get("/login/whoami", () => {
    return HttpResponse.json({ result: whoamiStub })
  }),

  http.get("/resource/provider", () => {
    return HttpResponse.json({ result: providerStubs })
  }),

  http.get("/resource/provider-links", () => {
    return HttpResponse.json({ result: providerLinkageStubs })
  }),

  http.get<{ provider: string; userId: string }>(
    "/resources/:provider/:userId",
    ({ params }) => {
      return HttpResponse.json({
        result: resourceStubs[params.provider][params.userId],
      })
    }
  ),
]
