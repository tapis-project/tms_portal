import { UserCircle2, LogOut, EllipsisVertical, LogIn } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { useAuth } from "@/tms-hooks/useAuth"
import { useWhoami } from "@/tms-hooks"

export function UserMenu() {
  const { data: isAuthenticated } = useAuth()
  const { data: whoami } = useWhoami({ enabled: !!isAuthenticated })
  if (!isAuthenticated || !whoami)
    return (
      <Button variant="outline" role="link" asChild>
        <a href="/login?idp_id=globus_idp&redirect_uri=https://tms-auth-service.tacc.cloud/">
          <LogIn /> Log In
        </a>
      </Button>
    )
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="h-12">
          <UserCircle2 className="size-6 text-muted-foreground" />
          <span className="hidden md:block">{whoami.name}</span>

          <EllipsisVertical className="size-5 text-muted-foreground md:ml-3" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuGroup>
          <DropdownMenuLabel>Provider: {whoami.organization}</DropdownMenuLabel>
          <DropdownMenuItem>
            <a href="#" className="flex items-center gap-2">
              <LogOut /> Log Out
            </a>
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
