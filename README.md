# Documentation
|  |  |
| :---         | :--- |
| API docs     | https://tms-trust-project.github.io/tms-live-docs |
| docs source  | https://github.com/tapis-project/tms-live-docs    |


# Build Instructions

## Prerequistes
You'll need a rust compiler and docker.  You'll need npm if you want to build the UI.

## Rust Build
```
cargo clean
cargo build ( or cargo build --release )
```

## Setup the database
```
./setupTmsPortalPg.sh (or if you want use a custom port ./setupTmsPortalPg.sh -p <portnum>)
```

## Build the UI (only if you want to run the UI locally)
At present, the UI has the url hard coded ... we probably need to change this.  But for now youll need to 
make a change in 2 files for it to work on your machine.  Here's the diff.  You can put it in a file, for
example patch.txt, and then git apply it.  Or, you could apply these changes manually.

`git apply patch.txt`

```
diff --git a/client/src/App.tsx b/client/src/App.tsx
index 6b18750..a351621 100644
--- a/client/src/App.tsx
+++ b/client/src/App.tsx
@@ -132,7 +132,7 @@ function App() {
           <div>
             Please{" "}
             <a
-              href="/login?idp_id=globus_idp&redirect_uri=https://tms-auth-service.tacc.cloud/"
+              href="/login?idp_id=globus_idp&redirect_uri=http://localhost:8080/"
               className="text-primary underline-offset-4 hover:underline"
             >
               log in
diff --git a/client/src/components/tms-ui/UserMenu.tsx b/client/src/components/tms-ui/UserMenu.tsx
index dad72b7..2278063 100644
--- a/client/src/components/tms-ui/UserMenu.tsx
+++ b/client/src/components/tms-ui/UserMenu.tsx
@@ -17,7 +17,7 @@ export function UserMenu() {
   if (!isAuthenticated || !whoami)
     return (
       <Button variant="outline" role="link" asChild>
-        <a href="/login?idp_id=globus_idp&redirect_uri=https://tms-auth-service.tacc.cloud/">
+        <a href="/login?idp_id=globus_idp&redirect_uri=http://localhost:8080/">
           <LogIn /> Log In
         </a>
       </Button>
```

`./setupClient.sh` 

Depending on the current working directory, you'll need to make sure the portal can see the ui code.
If you're running from the top level of the git repo, you can just create a symlink:
`ln -s ./client/dist/ ./dist`

## Create the schema
run the following command to create the schema from the tms_server project.  It will download the 
sqlx migration files directly from that project, and apply them.
```
deploy/migrate.sh 
```

deploy/migrate.sh -? will give all of the command line options.  There are default values for everything,
so you only need those parameters if you want to change something - for example the port.

The schema is created the first time that you run the portal.
```
export TMS_PORTAL_DB_PASSWORD=tms_password
export TMS_PORTAL_DB_HOST=localhost
```
If you changed the port, you may need this also:
```
export TMS_PORTAL_DB_PORT=<tms_db_port>
```

## Populate db with sample data
After editing the tms_portal_dev_config.sql template, use it to initialize the db.
```
cat tms_portal_dev_config.sql | docker exec -i tms_portal_postgres psql -U tms tms_db 
```
The tms_portal_dev_config.sql file is just an example, so you could also use a different file to initialize the 
database.  The initial setup will be different for each installation.

## Run
```
cargo run
```
Point the browser at http://localhost:8080/

Now the portal should be fully functional.  Some parts of this store cookies for later use.  If you want
to force a new login, you can go to globus.io, login, and logout.  This will force authentication
when you login via the tms portal.  For tacc resource provider, you can go to tacc.tapis.io, and from
the chrome dev tools, you can clear the cookies to force a new auth check.

