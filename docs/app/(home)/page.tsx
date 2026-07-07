import Link from 'next/link';
import { appName } from '@/lib/shared';

export default function HomePage() {
  return (
    <div className="flex flex-col justify-center text-center flex-1 items-center gap-4 p-8">
      <h1 className="text-4xl font-bold">{appName}</h1>
      <p className="text-lg text-muted-foreground max-w-xl">
        VTT tático peer-to-peer 3D low-poly — Rust + Bevy + WebRTC.
        Documentação da arquitetura, módulos e decisões de design.
      </p>
      <div className="flex gap-4 mt-4">
        <Link
          href="/docs/tabletop"
          className="px-6 py-3 rounded-xl bg-primary text-primary-foreground font-medium"
        >
          Ver documentação
        </Link>
        <Link
          href="https://github.com/JoaoHenriqueBarbosa/tabletop-p2p"
          className="px-6 py-3 rounded-xl border font-medium"
        >
          GitHub
        </Link>
      </div>
    </div>
  );
}
