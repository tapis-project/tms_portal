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
The schema is created the first time that you run the portal.
```
export TMS_PORTAL_DB_HOST=localhost
export TMS_PORTAL_DB_PASSWORD=tms_portal_password
```

If you changed the db password from the default dev configuration in the setupTmsPortalPg.sh script, 
you'll need to put in your own password instead of "tms_portal_password".

If you changed the default port for postgres:
```
export TMS_PORTAL_DB_PORT=<your port>
```

## quick test, to make sure the UI comes up
```
cargo run
```
Point the browser at http://localhost:8080/
NOTE:  While you'll be able to load the page, you will not be able to login, etc.  This needs keys
and configuration

## If desired, populate db with sample data
At this point everything is started up, but you'll need add identity providers, etc to make it work.  
This is all done in the db.
```
cat tms_portal_dev_config.sql | docker exec -i tms_portal_postgres psql -U tms_portal_user tms_portal_db
```
Now the portal should be fully functional.  Some parts of this store cookies for later use.  If you want
to force a new login, you can go to globus.io, login, and logout.  This will force authentication
when you login via the tms portal.  For tacc resource provider, you can go to tacc.tapis.io, and from
the chrome dev tools, you can clear the cookies to force a new auth check.

