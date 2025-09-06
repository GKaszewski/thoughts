import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";
import { User } from "lucide-react";

interface UserAvatarProps {
  src?: string | null;
  alt?: string | null;
  className?: string;
}

export function UserAvatar({ src, alt, className }: UserAvatarProps) {
  return (
    <Avatar className={cn("border-2 border-primary/50 shadow-md", className)}>
      {src && (
        <AvatarImage
          className="object-cover object-center"
          src={src}
          alt={alt ?? "User avatar"}
        />
      )}
      <AvatarFallback>
        <User className="h-5 w-5" />
      </AvatarFallback>
    </Avatar>
  );
}
