import {
  Viewer,
  Cesium3DTileset,
  Terrain,
  EllipsoidTerrainProvider,
  Rectangle,
  ImageryLayer,
  SingleTileImageryProvider,
  NearFarScalar,
} from "cesium";
import "cesium/Build/Cesium/Widgets/widgets.css";
import "./style.css";
import baseImageUrl from "./assets/world.topo.bathy.200406.3x5400x2700.jpg";

const params = new URLSearchParams(window.location.search);
const showGlobe = !params.has("noglobe");

const viewer = new Viewer("cesiumContainer", {
  terrain: new Terrain(new EllipsoidTerrainProvider()),
  baseLayer: showGlobe
    ? ImageryLayer.fromProviderAsync(
        SingleTileImageryProvider.fromUrl(baseImageUrl, {
          rectangle: Rectangle.fromDegrees(-180, -90, 180, 90),
        }),
      )
    : false,
  animation: false,
  timeline: false,
  baseLayerPicker: false,
  geocoder: false,
  scene3DOnly: true,
});

viewer.cesiumWidget.creditContainer.style.display = "none";

if (showGlobe) {
  // Fade the globe out as we approach it to avoid conflicting with the 3D Tiles layers
  viewer.scene.globe.depthTestAgainstTerrain = false;
  viewer.scene.globe.translucency.enabled = true;
  viewer.scene.globe.translucency.frontFaceAlphaByDistance = new NearFarScalar(
    1000.0,
    0.0,
    10000.0,
    0.5,
  );
} else {
  viewer.scene.globe.show = false;
}

async function loadLayer(name) {
  const tileset = await Cesium3DTileset.fromUrl(`/${name}`);
  tileset.cacheBytes = 1024 * (1 << 20);
  viewer.scene.primitives.add(tileset);
  return tileset;
}

async function loadData() {
  const names = (params.get("layers") ?? "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean);

  const tilesets = await Promise.all(names.map(loadLayer));
  if (tilesets.length) viewer.zoomTo(tilesets[0]);
}

loadData();
