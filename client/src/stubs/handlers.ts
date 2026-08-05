import { http, HttpResponse } from "msw"

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

const resourceStubs: Record<string, string>[] = [
  {
    id: "cffc01ff-a85d-437c-aebb-a42e05b817a9",
    name: "Stampede",
    description: "Stampede at TACC",
    provider_id: "tacc",
    provider_name: "TACC Resource Provider",
  },
  {
    id: "131d94fb-984e-4ce5-8f4f-adb965e7bc47",
    name: "Frontera",
    description: "Frontera at TACC",
    provider_id: "tacc",
    provider_name: "TACC Resource Provider",
  },
  {
    id: "7982b11e-f823-491e-9863-1096588f98f1",
    name: "Vista",
    description: "Vista at TACC",
    provider_id: "tacc",
    provider_name: "TACC Resource Provider",
  },
]

const providerLinksStub = {
  status: "200 OK",
  result: [
    {
      id: 8,
      tms_identity: "dbddf86d-a94e-4dc8-aa3e-19fe8a58fa7f@globus_idp",
      resource_provider_account: "jarosenb@tacc",
      resource_provider_uuid: "079164ad-daa9-4790-846d-73049848c41f",
      resource_provider_id: "tacc",
      resource_provider_name: "TACC Resource Provider",
      last_login: "2026-08-05T20:00:10.326842+00:00",
      enabled: false,
    },
  ],
}

export const handlers = [
  http.get("/login/whoami", () => {
    return HttpResponse.json({ result: whoamiStub })
  }),

  http.get("/resources/providers", () => {
    return HttpResponse.json({ result: providerStubs })
  }),
  http.get("/resources/providers/links", () => {
    return HttpResponse.json(providerLinksStub)
  }),
  http.delete<{ id: string }>("/resources/providers/links/:id", () => {
    return HttpResponse.json(providerLinksStub)
  }),

  http.get<{ provider: string; userId: string }>(
    "/resources/:provider_id/:provider_account_id",
    () => {
      return HttpResponse.json({
        result: resourceStubs,
      })
    }
  ),
]
