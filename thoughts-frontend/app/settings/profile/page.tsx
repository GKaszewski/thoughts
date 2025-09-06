import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { getMe } from "@/lib/api";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { EditProfileForm } from "@/components/edit-profile-form";

// This is a Server Component to fetch initial data
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
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Edit Profile</CardTitle>
          <CardDescription>
            Update your public profile information.
          </CardDescription>
        </CardHeader>
        <EditProfileForm currentUser={me} />
      </Card>
    </div>
  );
}
