"use client";

import { useEffect, useState } from "react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { PendingRequests } from "./pending-requests";
import { RemoteFollowers } from "./remote-followers";
import { RemoteFollowing } from "./remote-following";
import { getPendingFollowRequests } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";

export function FederationPanel() {
  const { token } = useAuth();
  const [pendingCount, setPendingCount] = useState(0);

  useEffect(() => {
    if (!token) return;
    getPendingFollowRequests(token)
      .then((r) => setPendingCount(r.length))
      .catch(() => {});
  }, [token]);

  return (
    <Tabs defaultValue="requests">
      <TabsList className="mb-4">
        <TabsTrigger value="requests">
          Requests
          {pendingCount > 0 && (
            <span className="ml-1.5 rounded-full bg-primary text-primary-foreground text-xs px-1.5 py-0.5">
              {pendingCount}
            </span>
          )}
        </TabsTrigger>
        <TabsTrigger value="followers">Followers</TabsTrigger>
        <TabsTrigger value="following">Following</TabsTrigger>
      </TabsList>
      <TabsContent value="requests">
        <PendingRequests />
      </TabsContent>
      <TabsContent value="followers">
        <RemoteFollowers />
      </TabsContent>
      <TabsContent value="following">
        <RemoteFollowing />
      </TabsContent>
    </Tabs>
  );
}
