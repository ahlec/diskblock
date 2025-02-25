# diskblock

Temporary steps:
1. Configure live/ directory in location referenced by plist (or change)
2. Copy daemon.plist to /Library/LaunchDaemons
3. `mv daemon.plist com.deitloff.alec.diskblock` (rename)
4. `sudo chown root:wheel daemon.plist`
5. You should now get a notification that it's been added to the background processes & it shows up in the system settings
6. `sudo launchctl load -w /Library/LaunchDaemons/com.deitloff.alec.diskblock.plist`
7. stdout and stderr files should now show up in your live directory
