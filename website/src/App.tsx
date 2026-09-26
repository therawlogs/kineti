import { HashRouter, Navigate, Route, Routes } from 'react-router-dom';
import { Layout } from './components/Layout';
import { Home } from './pages/Home';
import { Tools } from './pages/Tools';
import { DocDetail } from './pages/Docs';
import { Roadmap } from './pages/Roadmap';
import { Contribute } from './pages/Contribute';
import './index.css';

export function App(): JSX.Element {
  return (
    <HashRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/tools" element={<Tools />} />
          <Route path="/docs" element={<Navigate to="/tools" replace />} />
          <Route path="/docs/:slug" element={<DocDetail />} />
          <Route path="/roadmap" element={<Roadmap />} />
          <Route path="/contribute" element={<Contribute />} />
          <Route path="*" element={<Home />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}
