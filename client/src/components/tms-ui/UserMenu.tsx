import { UserCircle2, LogOut, EllipsisVertical } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"


export function UserMenu() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" className="h-12">
          <UserCircle2 className="size-6 text-muted-foreground" />
          <span className="hidden md:block">jarosenb</span>

          <EllipsisVertical className="size-5 text-muted-foreground md:ml-3" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuGroup>
          <DropdownMenuLabel>Provider: CILogon</DropdownMenuLabel>
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
