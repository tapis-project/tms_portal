
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { CheckIcon } from "lucide-react";

export function ResourceTable() {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Resource Name</TableHead>
          <TableHead>Resource Type</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Action</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow>
          <TableCell>Horizon</TableCell>
          <TableCell>Compute</TableCell>
          <TableCell>
            <Badge className="bg-green-600" variant="default">
              <CheckIcon />
              Linked
            </Badge>
          </TableCell>
          <TableCell>
            <Button variant="destructive">Unlink Resource </Button>
          </TableCell>
        </TableRow>
        <TableRow>
          <TableCell>Vista</TableCell>
          <TableCell>Compute</TableCell>
          <TableCell>
            <Badge variant="secondary">Unlinked</Badge>
          </TableCell>
          <TableCell>
            <Button variant="outline">Link Resource </Button>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  );
}

export function FacilityCard() {
  return (
    <Card className="m-3">
      <CardHeader>
        <CardTitle>TACC</CardTitle>
        <CardDescription>Texas Advanced Computing Center</CardDescription>
        <div className="flex items-center gap-2">
          <Badge variant="default" className="h-full bg-green-600">
            <CheckIcon /> Identity Linked
          </Badge>{" "}
          <span>
            <code>jrosenberg</code>
            <Button variant="link">(Unlink)</Button>
          </span>
        </div>
      </CardHeader>
      <CardContent>
        <h2 className="text-lg">Available Resources</h2>
        <ResourceTable />
      </CardContent>
    </Card>
  );
}

function App() {
  return (
    <div>
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        Trust Manager System
      </header>
      <div>
        <FacilityCard />
      </div>
    </div>
  );
}

export default App;
