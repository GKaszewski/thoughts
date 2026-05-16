// thoughts-frontend/components/api-key-list.tsx
"use client";

import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { toast } from "sonner";
import { useAuth } from "@/hooks/use-auth";
import {
  ApiKey,
  CreateApiKeySchema,
  createApiKey,
  deleteApiKey,
} from "@/lib/api";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Form,
  FormField,
  FormItem,
  FormLabel,
  FormControl,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Copy, KeyRound, Plus, Trash2 } from "lucide-react";
import { format } from "date-fns";

interface ApiKeyListProps {
  initialApiKeys: ApiKey[];
}

export function ApiKeyList({ initialApiKeys }: ApiKeyListProps) {
  const [keys, setKeys] = useState<ApiKey[]>(initialApiKeys);
  const [newKey, setNewKey] = useState<string | null>(null);
  const { token } = useAuth();

  const form = useForm<z.infer<typeof CreateApiKeySchema>>({
    resolver: zodResolver(CreateApiKeySchema),
    defaultValues: { name: "" },
  });

  async function onSubmit(values: z.infer<typeof CreateApiKeySchema>) {
    if (!token) return;
    try {
      const newKeyResponse = await createApiKey(values, token);
      setKeys((prev) => [...prev, newKeyResponse]);
      setNewKey(newKeyResponse.key ?? null);
      form.reset();
      toast.success("API Key created successfully.");
    } catch {
      toast.error("Failed to create API key.");
    }
  }

  const handleDelete = async (keyId: string) => {
    if (!token) return;
    try {
      await deleteApiKey(keyId, token);
      setKeys((prev) => prev.filter((key) => key.id !== keyId));
      toast.success("API Key deleted successfully.");
    } catch {
      toast.error("Failed to delete API key.");
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    toast.success("Key copied to clipboard!");
  };

  return (
    <div className="space-y-8">
      <Card>
        <CardHeader>
          <CardTitle>Existing Keys</CardTitle>
          <CardDescription>
            These are the API keys associated with your account.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {keys.length > 0 ? (
            <ul className="divide-y">
              {keys.map((key) => (
                <li
                  key={key.id}
                  className="flex items-center justify-between p-4 glass-effect rounded-md shadow-fa-sm glossy-effect bottom"
                >
                  <div className="flex items-center gap-4">
                    <KeyRound className="h-6 w-6 text-muted-foreground" />
                    <div>
                      <p className="font-semibold">{key.name}</p>
                      <p className="text-sm text-muted-foreground">
                        {`Created on ${format(key.createdAt, "PPP")}`}
                      </p>
                      <p className="text-xs font-mono text-muted-foreground mt-1">
                        {key.id}
                      </p>
                    </div>
                  </div>
                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button variant="outline" size="icon">
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>Are you sure?</AlertDialogTitle>
                        <AlertDialogDescription>
                          This will permanently delete the key &quot;{key.name}
                          &quot;. This action cannot be undone.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction onClick={() => handleDelete(key.id)}>
                          Delete
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-sm text-muted-foreground">
              You have no API keys.
            </p>
          )}
        </CardContent>
      </Card>

      {/* Display New Key */}
      {newKey && (
        <Card>
          <CardHeader>
            <CardTitle>New API Key Generated</CardTitle>
            <CardDescription>
              Please copy this key and store it securely. You will not be able
              to see it again.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex items-center gap-4">
            <Input readOnly value={newKey} className="font-mono" />
            <Button
              size="icon"
              variant="outline"
              onClick={() => copyToClipboard(newKey)}
            >
              <Copy className="h-4 w-4" />
            </Button>
          </CardContent>
          <CardFooter>
            <Button onClick={() => setNewKey(null)}>Done</Button>
          </CardFooter>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Create New API Key</CardTitle>
          <CardDescription>
            Give your new key a descriptive name.
          </CardDescription>
        </CardHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)}>
            <CardContent>
              <FormField
                name="name"
                control={form.control}
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Key Name</FormLabel>
                    <FormControl>
                      <Input placeholder="e.g., My Cool App" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </CardContent>
            <CardFooter className="px-6 py-4">
              <Button type="submit" disabled={form.formState.isSubmitting}>
                <Plus className="mr-2 h-4 w-4" />
                {form.formState.isSubmitting ? "Creating..." : "Create Key"}
              </Button>
            </CardFooter>
          </form>
        </Form>
      </Card>
    </div>
  );
}
