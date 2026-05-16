"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Me, UpdateProfileSchema } from "@/lib/api";
import { updateProfile } from "@/app/actions/profile";
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
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";

interface EditProfileFormProps {
  currentUser: Me;
}

export function EditProfileForm({ currentUser }: EditProfileFormProps) {

  const form = useForm<z.infer<typeof UpdateProfileSchema>>({
    resolver: zodResolver(UpdateProfileSchema),
    defaultValues: {
      displayName: currentUser.displayName ?? undefined,
      bio: currentUser.bio ?? undefined,
      avatarUrl: currentUser.avatarUrl ?? undefined,
      headerUrl: currentUser.headerUrl ?? undefined,
      customCss: currentUser.customCss ?? undefined,
    },
  });

  async function onSubmit(values: z.infer<typeof UpdateProfileSchema>) {
    toast.info("Updating your profile...");
    try {
      await updateProfile(currentUser.username, values);
      toast.success("Profile updated successfully!");
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
