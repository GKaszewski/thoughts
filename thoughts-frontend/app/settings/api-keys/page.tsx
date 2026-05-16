import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { getApiKeys } from "@/lib/api";
import { ApiKeyList } from "@/components/api-keys-list";

export default async function ApiKeysPage() {
  const token = (await cookies()).get("auth_token")?.value;
  if (!token) {
    redirect("/login");
  }

  const initialApiKeys = await getApiKeys(token).catch(() => ({
    keys: [],
  }));

  return (
    <div className="space-y-6">
      <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4">
        <h3 className="text-lg font-medium">API Keys</h3>
        <p className="text-sm text-muted-foreground">
          Manage API keys for third-party applications.
        </p>
      </div>
      <ApiKeyList initialApiKeys={initialApiKeys.keys} />
    </div>
  );
}
