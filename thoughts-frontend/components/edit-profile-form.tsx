"use client";

import { useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Me, UpdateProfileSchema, updateProfile, uploadAvatar, uploadBanner } from "@/lib/api";
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
import { UserAvatar } from "@/components/user-avatar";
import { CssEditor } from "@/components/css-editor";
import { Camera, ImagePlus, Loader2 } from "lucide-react";

interface EditProfileFormProps {
  currentUser: Me;
  token: string;
}

export function EditProfileForm({ currentUser, token }: EditProfileFormProps) {
  const router = useRouter();
  const avatarInputRef = useRef<HTMLInputElement>(null);
  const bannerInputRef = useRef<HTMLInputElement>(null);

  const [avatarSrc, setAvatarSrc] = useState(currentUser.avatarUrl ?? null);
  const [bannerSrc, setBannerSrc] = useState(currentUser.headerUrl ?? null);
  const [uploadingAvatar, setUploadingAvatar] = useState(false);
  const [uploadingBanner, setUploadingBanner] = useState(false);

  async function handleAvatarChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploadingAvatar(true);
    try {
      const updated = await uploadAvatar(file, token);
      setAvatarSrc(updated.avatarUrl ?? null);
      router.refresh();
      toast.success("Avatar updated");
    } catch {
      toast.error("Failed to upload avatar");
    } finally {
      setUploadingAvatar(false);
      e.target.value = "";
    }
  }

  async function handleBannerChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploadingBanner(true);
    try {
      const updated = await uploadBanner(file, token);
      setBannerSrc(updated.headerUrl ?? null);
      router.refresh();
      toast.success("Banner updated");
    } catch {
      toast.error("Failed to upload banner");
    } finally {
      setUploadingBanner(false);
      e.target.value = "";
    }
  }

  const form = useForm<z.infer<typeof UpdateProfileSchema>>({
    resolver: zodResolver(UpdateProfileSchema),
    defaultValues: {
      displayName: currentUser.displayName ?? undefined,
      bio: currentUser.bio ?? undefined,
      customCss: currentUser.customCss ?? undefined,
    },
  });

  async function onSubmit(values: z.infer<typeof UpdateProfileSchema>) {
    if (!token) return;
    toast.info("Updating your profile...");
    try {
      await updateProfile(values, token);
      router.refresh();
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

            {/* Banner */}
            <div>
              <p className="text-sm font-medium mb-2">Banner</p>
              <div
                className="relative h-32 rounded-md bg-muted overflow-hidden cursor-pointer group"
                onClick={() => !uploadingBanner && bannerInputRef.current?.click()}
              >
                {bannerSrc ? (
                  <img
                    src={bannerSrc}
                    alt="Banner"
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center text-muted-foreground text-sm">
                    No banner
                  </div>
                )}
                <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                  {uploadingBanner ? (
                    <Loader2 className="text-white h-6 w-6 animate-spin" />
                  ) : (
                    <ImagePlus className="text-white h-6 w-6" />
                  )}
                </div>
              </div>
              <input
                ref={bannerInputRef}
                type="file"
                accept="image/*"
                className="hidden"
                onChange={handleBannerChange}
                disabled={uploadingBanner}
              />
            </div>

            {/* Avatar */}
            <div>
              <p className="text-sm font-medium mb-2">Avatar</p>
              <div className="flex items-center gap-4">
                <div
                  className="relative w-20 h-20 rounded-full cursor-pointer group shrink-0"
                  onClick={() => !uploadingAvatar && avatarInputRef.current?.click()}
                >
                  <UserAvatar
                    src={avatarSrc}
                    alt={currentUser.displayName}
                    className="w-full h-full"
                  />
                  <div className="absolute inset-0 rounded-full bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                    {uploadingAvatar ? (
                      <Loader2 className="text-white h-4 w-4 animate-spin" />
                    ) : (
                      <Camera className="text-white h-4 w-4" />
                    )}
                  </div>
                </div>
                <p className="text-sm text-muted-foreground">
                  Click to upload · JPEG, PNG, GIF, WebP, AVIF · max 5 MB
                </p>
              </div>
              <input
                ref={avatarInputRef}
                type="file"
                accept="image/*"
                className="hidden"
                onChange={handleAvatarChange}
                disabled={uploadingAvatar}
              />
            </div>

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
              name="customCss"
              control={form.control}
              render={({ field }) => (
                <FormItem>
                  <FormControl>
                    <CssEditor
                      value={field.value ?? ""}
                      onChange={field.onChange}
                      username={currentUser.username}
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
