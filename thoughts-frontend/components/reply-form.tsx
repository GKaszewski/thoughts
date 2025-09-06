// components/reply-form.tsx
"use client";

import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormField,
  FormItem,
  FormControl,
  FormMessage,
} from "@/components/ui/form";
import { Textarea } from "@/components/ui/textarea";
import { CreateThoughtSchema, createThought } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { toast } from "sonner";

interface ReplyFormProps {
  parentThoughtId: string;
  onReplySuccess: () => void; // A callback to close the form after success
}

export function ReplyForm({ parentThoughtId, onReplySuccess }: ReplyFormProps) {
  const router = useRouter();
  const { token } = useAuth();

  const form = useForm<z.infer<typeof CreateThoughtSchema>>({
    resolver: zodResolver(CreateThoughtSchema),
    defaultValues: {
      content: "",
      replyToId: parentThoughtId,
      visibility: "Public", // Replies default to Public
    },
  });

  async function onSubmit(values: z.infer<typeof CreateThoughtSchema>) {
    if (!token) {
      toast.error("You must be logged in to reply.");
      return;
    }

    try {
      await createThought(values, token);
      toast.success("Your reply has been posted!");
      form.reset();
      onReplySuccess(); // Call the callback
      router.refresh(); // Refresh the page to show the new reply
    } catch (err) {
      toast.error("Failed to post reply. Please try again.");
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-2 p-4">
        <FormField
          control={form.control}
          name="content"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <Textarea
                  placeholder="Post your reply..."
                  className="resize-none bg-white glass-effect glossy-efect bottom shadow-fa-sm"
                  {...field}
                />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <div className="flex justify-end gap-2">
          <Button
            type="button"
            variant="ghost"
            onClick={onReplySuccess} // Close button
          >
            Cancel
          </Button>
          <Button type="submit" disabled={form.formState.isSubmitting}>
            {form.formState.isSubmitting ? "Replying..." : "Reply"}
          </Button>
        </div>
      </form>
    </Form>
  );
}
