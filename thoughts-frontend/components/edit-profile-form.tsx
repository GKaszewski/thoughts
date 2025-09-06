"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useRouter } from "next/navigation";
import { useAuth } from "@/hooks/use-auth";
import { Me, UpdateProfileSchema, updateProfile } from "@/lib/api";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardFooter } from "@/components/ui/card";
import {
  Form,
  FormField,
  FormItem,
  FormLabel,
  FormControl,
  FormMessage,
  FormDescription,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

interface EditProfileFormProps {
  currentUser: Me;
}

export function EditProfileForm({ currentUser }: EditProfileFormProps) {
  const router = useRouter();
  const { token } = useAuth();

  const form = useForm<z.infer<typeof UpdateProfileSchema>>({
    resolver: zodResolver(UpdateProfileSchema),
    defaultValues: {
      displayName: currentUser.displayName ?? undefined,
      bio: currentUser.bio ?? undefined,
      avatarUrl: currentUser.avatarUrl ?? undefined,
      headerUrl: currentUser.headerUrl ?? undefined,
      customCss: currentUser.customCss ?? undefined,
      topFriends: currentUser.topFriends ?? [],
    },
  });

  async function onSubmit(values: z.infer<typeof UpdateProfileSchema>) {
    if (!token) return;
    toast.info("Updating your profile...");
    try {
      await updateProfile(values, token);
      toast.success("Profile updated successfully!");
      // Redirect to the profile page to see the changes
      router.push(`/users/${currentUser.username}`);
      router.refresh(); // Ensure fresh data is loaded
    } catch (err) {
      toast.error(`Failed to update profile. ${err}`);
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Card>
          <CardContent className="space-y-6 pt-6">
            <FormField
              name="displayName"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Display Name</FormLabel>
                  <FormControl>
                    <Input placeholder="Your display name" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              name="bio"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Bio</FormLabel>
                  <FormControl>
                    <Textarea placeholder="Tell us about yourself" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              name="avatarUrl"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Avatar URL</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="https://example.com/avatar.png"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              name="headerUrl"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Header URL</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="https://example.com/header.jpg"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              name="customCss"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Custom CSS</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="body { font-family: 'Comic Sans MS'; }"
                      rows={5}
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              name="topFriends"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Top Friends</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="username1, username2, ..."
                      {...field}
                      onChange={(e) =>
                        field.onChange(
                          e.target.value.split(",").map((s) => s.trim())
                        )
                      }
                    />
                  </FormControl>
                  <FormDescription>
                    A comma-separated list of usernames.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
          </CardContent>
          <CardFooter className="border-t px-6 py-4">
            <Button type="submit" disabled={form.formState.isSubmitting}>
              {form.formState.isSubmitting ? "Saving..." : "Save Changes"}
            </Button>
          </CardFooter>
        </Card>
      </form>
    </Form>
  );
}
