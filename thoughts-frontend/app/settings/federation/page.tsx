import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { FederationPanel } from "@/components/federation/federation-panel";
import { MigrationSettings } from "@/components/federation/migration-settings";

export default async function FederationSettingsPage() {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) {
    redirect("/login");
  }

  return (
    <div className="space-y-6">
      <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4">
        <h3 className="text-lg font-medium">Federation</h3>
        <p className="text-sm text-muted-foreground">
          Manage remote follow requests, followers, and accounts you follow on
          other instances.
        </p>
      </div>
      <FederationPanel />
      <MigrationSettings />
    </div>
  );
}
