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
const providerLinkageStubs = [
  {
    providerId: "tacc",
    tmsIdentity: "jarosenb",
    providerIdentity: "jarosenb",
  },
]

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

export const handlers = [
  http.get("/login/whoami", () => {
    return HttpResponse.json({ result: whoamiStub })
  }),

  http.get("/resources/providers", () => {
    return HttpResponse.json({ result: providerStubs })
  }),

  http.get("/resource/provider-links", () => {
    return HttpResponse.json({ result: providerLinkageStubs })
  }),

  http.get<{ provider: string; userId: string }>("/resources", () => {
    return HttpResponse.json({
      result: resourceStubs,
    })
  }),
]
