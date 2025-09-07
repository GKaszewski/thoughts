"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Form,
  FormField,
  FormItem,
  FormControl,
  FormMessage,
} from "@/components/ui/form";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { CreateThoughtSchema, createThought } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { toast } from "sonner";
import { Globe, Lock, Users } from "lucide-react";
import { useState } from "react";
import { Confetti } from "./confetti";

export function PostThoughtForm() {
  const router = useRouter();
  const { token } = useAuth();
  const [showConfetti, setShowConfetti] = useState(false);

  const form = useForm<z.infer<typeof CreateThoughtSchema>>({
    resolver: zodResolver(CreateThoughtSchema),
    defaultValues: { content: "", visibility: "Public" },
  });

  async function onSubmit(values: z.infer<typeof CreateThoughtSchema>) {
    if (!token) {
      toast.error("You must be logged in to post.");
      return;
    }

    try {
      await createThought(values, token);
      toast.success("Your thought has been posted!");
      setShowConfetti(true);
      form.reset();
      router.refresh(); // This is the key to updating the feed
    } catch {
      toast.error("Failed to post thought. Please try again.");
    }
  }

  return (
    <>
      <Confetti fire={showConfetti} onComplete={() => setShowConfetti(false)} />
      <Card>
        <CardContent className="p-4">
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              <FormField
                control={form.control}
                name="content"
                render={({ field }) => (
                  <FormItem>
                    <FormControl>
                      <Textarea
                        placeholder="What's on your mind?"
                        className="resize-none"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <div className="flex justify-between items-center">
                <FormField
                  control={form.control}
                  name="visibility"
                  render={({ field }) => (
                    <Select
                      onValueChange={field.onChange}
                      defaultValue={field.value}
                    >
                      <FormControl>
                        <SelectTrigger className="w-[150px]">
                          <SelectValue placeholder="Visibility" />
                        </SelectTrigger>
                      </FormControl>
                      <SelectContent>
                        <SelectItem value="Public">
                          <div className="flex items-center gap-2">
                            <Globe className="h-4 w-4" /> Public
                          </div>
                        </SelectItem>
                        <SelectItem value="FriendsOnly">
                          <div className="flex items-center gap-2">
                            <Users className="h-4 w-4" /> Friends Only
                          </div>
                        </SelectItem>
                        <SelectItem value="Private">
                          <div className="flex items-center gap-2">
                            <Lock className="h-4 w-4" /> Private
                          </div>
                        </SelectItem>
                      </SelectContent>
                    </Select>
                  )}
                />
                <Button type="submit" disabled={form.formState.isSubmitting}>
                  {form.formState.isSubmitting ? "Posting..." : "Post Thought"}
                </Button>
              </div>
            </form>
          </Form>
        </CardContent>
      </Card>
    </>
  );
}
