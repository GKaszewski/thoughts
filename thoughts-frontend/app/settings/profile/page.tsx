// app/settings/profile/page.tsx
import type { Metadata } from "next";
import { cookies } from "next/headers";

export const metadata: Metadata = {
  title: "Edit profile",
  description: "Update your Thoughts profile",
};
import { redirect } from "next/navigation";
import { getMe } from "@/lib/api";
import { EditProfileForm } from "@/components/edit-profile-form";

export default async function EditProfilePage() {
  const token = (await cookies()).get("auth_token")?.value;

  if (!token) {
    redirect("/login");
  }

  const me = await getMe(token).catch(() => null);

  if (!me) {
    redirect("/login");
  }

  return (
    <div className="space-y-6 ">
      <div className="glass-effect glossy-effect bottom rounded-md shadow-fa-lg p-4">
        <h3 className="text-lg font-medium">Profile</h3>
        <p className="text-sm text-muted-foreground">
          This is how others will see you on the site.
        </p>
      </div>
      <EditProfileForm currentUser={me} token={token} />
    </div>
  );
}
