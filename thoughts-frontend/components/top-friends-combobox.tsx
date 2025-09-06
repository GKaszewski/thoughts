"use client";

import * as React from "react";
import { Check, ChevronsUpDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { getFriends, User } from "@/lib/api";
import { useAuth } from "@/hooks/use-auth";
import { Skeleton } from "./ui/skeleton";

interface TopFriendsComboboxProps {
  value: string[];
  onChange: (value: string[]) => void;
}

export function TopFriendsCombobox({
  value,
  onChange,
}: TopFriendsComboboxProps) {
  const [open, setOpen] = React.useState(false);
  const [friends, setFriends] = React.useState<User[]>([]);
  const [isLoading, setIsLoading] = React.useState(true);
  const { token } = useAuth();

  React.useEffect(() => {
    if (token) {
      getFriends(token)
        .then((data) => setFriends(data.users))
        .catch(() => console.error("Failed to fetch friends"))
        .finally(() => setIsLoading(false));
    } else {
      setIsLoading(false);
    }
  }, [token]);

  if (isLoading) {
    return <Skeleton className="h-10 w-full" />;
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-full justify-between"
        >
          {value.length > 0
            ? `${value.length} friend(s) selected`
            : "Select up to 8 friends..."}
          <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-full p-0">
        <Command>
          <CommandInput placeholder="Search friends..." />
          <CommandList>
            <CommandEmpty>No friends found.</CommandEmpty>
            <CommandGroup>
              {friends.map((friend) => (
                <CommandItem
                  key={friend.id}
                  value={friend.username}
                  onSelect={(currentValue) => {
                    const newValue = value.includes(currentValue)
                      ? value.filter((v) => v !== currentValue)
                      : [...value, currentValue];

                    if (newValue.length <= 8) {
                      onChange(newValue);
                    }
                  }}
                >
                  <Check
                    className={cn(
                      "mr-2 h-4 w-4",
                      value.includes(friend.username)
                        ? "opacity-100"
                        : "opacity-0"
                    )}
                  />
                  {friend.username}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
