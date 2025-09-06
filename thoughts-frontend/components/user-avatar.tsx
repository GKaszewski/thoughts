import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { User } from "lucide-react";

interface UserAvatarProps {
  src?: string | null;
  alt?: string | null;
}

export function UserAvatar({ src, alt }: UserAvatarProps) {
  return (
    <Avatar>
      {src && <AvatarImage src={src} alt={alt ?? "User avatar"} />}
      <AvatarFallback>
        <User className="h-5 w-5" />
      </AvatarFallback>
    </Avatar>
  );
}
