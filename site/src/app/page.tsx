import ScrollWorld from '@/components/ScrollWorld';
import JsonLd from '@/components/JsonLd';
import Features from '@/components/Features';
import Faq from '@/components/Faq';
import Footer from '@/components/Footer';

export default function Home() {
  return (
    <>
      <JsonLd />
      <span id="top" />
      <ScrollWorld />
      <Features />
      <Faq />
      <Footer />
    </>
  );
}
