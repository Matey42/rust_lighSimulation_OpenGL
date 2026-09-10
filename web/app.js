const canvas = document.querySelector('#scene');
window.addEventListener('error', event => {
  const loading = document.querySelector('#loading');
  const screen = document.querySelector('#error');
  if (loading) loading.hidden = true;
  if (screen) {
    screen.hidden = false;
    const title = screen.querySelector('h2');
    const detail = screen.querySelector('p');
    if (title) title.textContent = 'The renderer could not be initialized.';
    if (detail) detail.textContent = event.message || 'Check the browser console for details.';
  }
});
const gl = canvas.getContext('webgl2', { antialias: true, alpha: false, powerPreference: 'high-performance' });

if (!gl) {
  document.querySelector('#loading').hidden = true;
  document.querySelector('#error').hidden = false;
  throw new Error('WebGL2 is not available');
}

// Small, dependency-free vector and column-major matrix toolkit.
const V3 = {
  add: (a,b) => [a[0]+b[0],a[1]+b[1],a[2]+b[2]],
  sub: (a,b) => [a[0]-b[0],a[1]-b[1],a[2]-b[2]],
  scale: (a,s) => [a[0]*s,a[1]*s,a[2]*s],
  dot: (a,b) => a[0]*b[0]+a[1]*b[1]+a[2]*b[2],
  cross: (a,b) => [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]],
  length: a => Math.hypot(a[0],a[1],a[2]),
  norm(a){ const l=this.length(a)||1; return this.scale(a,1/l); }
};

const M4 = {
  identity: () => new Float32Array([1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1]),
  multiply(a,b){
    const o=new Float32Array(16);
    for(let c=0;c<4;c++) for(let r=0;r<4;r++) o[c*4+r]=a[r]*b[c*4]+a[4+r]*b[c*4+1]+a[8+r]*b[c*4+2]+a[12+r]*b[c*4+3];
    return o;
  },
  perspective(fovy,aspect,near,far){const f=1/Math.tan(fovy/2),nf=1/(near-far);return new Float32Array([f/aspect,0,0,0, 0,f,0,0, 0,0,(far+near)*nf,-1, 0,0,2*far*near*nf,0]);},
  lookAt(eye,target,up=[0,1,0]){const z=V3.norm(V3.sub(eye,target)),x=V3.norm(V3.cross(up,z)),y=V3.cross(z,x);return new Float32Array([x[0],y[0],z[0],0,x[1],y[1],z[1],0,x[2],y[2],z[2],0,-V3.dot(x,eye),-V3.dot(y,eye),-V3.dot(z,eye),1]);},
  translation(v){const m=this.identity();m[12]=v[0];m[13]=v[1];m[14]=v[2];return m;},
  scaling(v){const m=this.identity();m[0]=v[0];m[5]=v[1];m[10]=v[2];return m;},
  rotationX(a){const c=Math.cos(a),s=Math.sin(a);return new Float32Array([1,0,0,0,0,c,s,0,0,-s,c,0,0,0,0,1]);},
  rotationY(a){const c=Math.cos(a),s=Math.sin(a);return new Float32Array([c,0,-s,0,0,1,0,0,s,0,c,0,0,0,0,1]);},
  rotationZ(a){const c=Math.cos(a),s=Math.sin(a);return new Float32Array([c,s,0,0,-s,c,0,0,0,0,1,0,0,0,0,1]);},
  lookRotation(direction){
    const forward=V3.norm(direction),worldUp=Math.abs(forward[1])>.99?[0,0,1]:[0,1,0];
    const right=V3.norm(V3.cross(forward,worldUp)),up=V3.cross(right,forward);
    return new Float32Array([right[0],right[1],right[2],0,up[0],up[1],up[2],0,-forward[0],-forward[1],-forward[2],0,0,0,0,1]);
  },
  compose(position,rotation=[0,0,0],scale=[1,1,1]){return this.multiply(this.translation(position),this.multiply(this.rotationY(rotation[1]),this.multiply(this.rotationX(rotation[0]),this.multiply(this.rotationZ(rotation[2]),this.scaling(scale)))));},
  transformPoint(m,v){return [m[0]*v[0]+m[4]*v[1]+m[8]*v[2]+m[12],m[1]*v[0]+m[5]*v[1]+m[9]*v[2]+m[13],m[2]*v[0]+m[6]*v[1]+m[10]*v[2]+m[14]];},
  normal3(m){
    const a00=m[0],a01=m[1],a02=m[2],a10=m[4],a11=m[5],a12=m[6],a20=m[8],a21=m[9],a22=m[10];
    const b01=a22*a11-a12*a21,b11=-a22*a10+a12*a20,b21=a21*a10-a11*a20;let d=a00*b01+a01*b11+a02*b21;d=d?1/d:1;
    const inverse=[b01*d,(-a22*a01+a02*a21)*d,(a12*a01-a02*a11)*d,b11*d,(a22*a00-a02*a20)*d,(-a12*a00+a02*a10)*d,b21*d,(-a21*a00+a01*a20)*d,(a11*a00-a01*a10)*d];
    return new Float32Array([inverse[0],inverse[3],inverse[6],inverse[1],inverse[4],inverse[7],inverse[2],inverse[5],inverse[8]]);
  }
};

const vertexSource = `#version 300 es
precision highp float;
precision highp int;
layout(location=0) in vec3 aPosition;
layout(location=1) in vec3 aNormal;
layout(location=2) in vec2 aUv;
layout(location=3) in vec3 aTangent;
layout(location=4) in vec3 aBitangent;
uniform mat4 uModel, uViewProjection;
uniform mat3 uNormalMatrix;
uniform vec3 uCamera, uBaseColor, uSpecular;
uniform float uShininess, uAmbient;
uniform int uUseNormalMap;
uniform highp sampler2D uNormalMap;
uniform int uLightCount, uLightType[5];
uniform vec3 uLightPosition[5], uLightDirection[5], uLightColor[5];
uniform float uLightIntensity[5], uInnerCutoff[5], uOuterCutoff[5];
out vec3 vWorldPosition, vNormal, vTangent, vBitangent, vGouraud, vLocalPosition;
out vec2 vUv;
vec3 illuminate(vec3 p, vec3 n){
  vec3 color=vec3(0.015)+uBaseColor*uAmbient*.12;
  vec3 viewDir=normalize(uCamera-p);
  for(int i=0;i<5;i++){
    if(i>=uLightCount||uLightIntensity[i]<=0.0) continue;
    vec3 lightDir; float attenuation=1.0; float spot=1.0;
    if(uLightType[i]==2){lightDir=normalize(-uLightDirection[i]);}
    else {vec3 toLight=uLightPosition[i]-p;float dist=length(toLight);lightDir=toLight/max(dist,.001);attenuation=1.0/(1.0+.09*dist+.032*dist*dist);}
    if(uLightType[i]==1){float theta=dot(-lightDir,normalize(uLightDirection[i]));spot=smoothstep(uOuterCutoff[i],uInnerCutoff[i],theta);}
    float diff=max(dot(n,lightDir),0.0);
    vec3 halfDir=normalize(lightDir+viewDir);
    float spec=pow(max(dot(n,halfDir),0.0),uShininess)*step(0.001,diff);
    color+=(uBaseColor*diff+uSpecular*spec)*uLightColor[i]*uLightIntensity[i]*attenuation*spot;
  }
  return color;
}
void main(){vec4 world=uModel*vec4(aPosition,1.0);vWorldPosition=world.xyz;vLocalPosition=aPosition;vNormal=normalize(uNormalMatrix*aNormal);vTangent=normalize(uNormalMatrix*aTangent);vBitangent=normalize(uNormalMatrix*aBitangent);vUv=aUv;vec3 vertexNormal=vNormal;if(uUseNormalMap!=0){vec3 map=textureLod(uNormalMap,vUv,0.0).xyz*2.0-1.0;map.xy*=1.45;vertexNormal=normalize(mat3(vTangent,vBitangent,vNormal)*map);}vGouraud=illuminate(world.xyz,vertexNormal);gl_Position=uViewProjection*world;}`;

const fragmentSource = `#version 300 es
precision highp float;
precision highp int;
in vec3 vWorldPosition, vNormal, vTangent, vBitangent, vGouraud, vLocalPosition;
in vec2 vUv;
uniform vec3 uCamera, uBaseColor, uSpecular, uFogColor, uEmissiveColor;
uniform float uShininess, uAmbient, uFogDensity, uAlpha;
uniform bool uPhong, uFogEnabled, uEmissive, uCone;
uniform int uUseNormalMap;
uniform highp sampler2D uNormalMap;
uniform int uLightCount, uLightType[5];
uniform vec3 uLightPosition[5], uLightDirection[5], uLightColor[5];
uniform float uLightIntensity[5], uInnerCutoff[5], uOuterCutoff[5];
out vec4 fragColor;
vec3 surfaceNormal(){
  vec3 n=normalize(vNormal);if(uUseNormalMap==0)return n;
  vec3 map=texture(uNormalMap,vUv).xyz*2.0-1.0;map.xy*=1.45;
  return normalize(mat3(normalize(vTangent),normalize(vBitangent),n)*map);
}
vec3 illuminate(vec3 p,vec3 n){
  vec3 color=vec3(0.015)+uBaseColor*uAmbient*.12;vec3 viewDir=normalize(uCamera-p);
  for(int i=0;i<5;i++){
    if(i>=uLightCount||uLightIntensity[i]<=0.0)continue;vec3 lightDir;float attenuation=1.0;float spot=1.0;
    if(uLightType[i]==2){lightDir=normalize(-uLightDirection[i]);}
    else{vec3 toLight=uLightPosition[i]-p;float dist=length(toLight);lightDir=toLight/max(dist,.001);attenuation=1.0/(1.0+.09*dist+.032*dist*dist);}
    if(uLightType[i]==1){float theta=dot(-lightDir,normalize(uLightDirection[i]));spot=smoothstep(uOuterCutoff[i],uInnerCutoff[i],theta);}
    float diff=max(dot(n,lightDir),0.0);vec3 halfDir=normalize(lightDir+viewDir);float spec=pow(max(dot(n,halfDir),0.0),uShininess)*step(0.001,diff);
    color+=(uBaseColor*diff+uSpecular*spec)*uLightColor[i]*uLightIntensity[i]*attenuation*spot;
  }return color;
}
void main(){vec3 color=uEmissive?uEmissiveColor:(uPhong?illuminate(vWorldPosition,surfaceNormal()):vGouraud);float alpha=uAlpha;if(uCone){float along=clamp(-vLocalPosition.z,0.0,1.0);alpha*=1.0-smoothstep(.12,1.0,along);}if(uFogEnabled){float d=distance(uCamera,vWorldPosition);float f=clamp(exp(-pow(uFogDensity*d,2.0)),0.0,1.0);color=mix(uFogColor,color,f);alpha*=f;}fragColor=vec4(color,alpha);}`;

const gridVertexSource = `#version 300 es
precision highp float;layout(location=0)in vec3 aPosition;uniform mat4 uViewProjection;out vec3 vWorld;void main(){vWorld=aPosition;gl_Position=uViewProjection*vec4(aPosition,1.0);}`;
const gridFragmentSource = `#version 300 es
precision highp float;precision highp int;in vec3 vWorld;uniform vec3 uCamera,uFogColor;uniform float uFogDensity,uDay,uGridSize,uSubGridSize,uFadeRadius,uLineWidth;uniform bool uFogEnabled;uniform int uNumSpots;uniform vec3 uSpotPosition[4],uSpotDirection[4],uSpotColor[4];uniform float uSpotInner[4],uSpotOuter[4];out vec4 fragColor;
float line(vec2 p,float spacing,float width){vec2 cell=abs(fract(p/spacing-.5)-.5)*spacing;vec2 aa=fwidth(p);vec2 ink=1.0-smoothstep(vec2(width*.5),vec2(width*.5)+aa,cell);return max(ink.x,ink.y);}
vec3 spotLighting(){vec3 total=vec3(0);for(int i=0;i<4;i++){if(i>=uNumSpots)break;vec3 ray=vWorld-uSpotPosition[i];float dist=length(ray);vec3 fromLight=ray/max(dist,.001);float cone=smoothstep(uSpotOuter[i],uSpotInner[i],dot(fromLight,normalize(uSpotDirection[i])));float diffuse=max(dot(vec3(0,1,0),-fromLight),0.0);float attenuation=1.0/(1.0+.09*dist+.032*dist*dist);total+=uSpotColor[i]*cone*diffuse*attenuation;}return total;}
void main(){float minor=line(vWorld.xz,uSubGridSize,uLineWidth*.6),major=line(vWorld.xz,uGridSize,uLineWidth);float ax=1.0-smoothstep(0.0,fwidth(vWorld.z)+uLineWidth*1.5,abs(vWorld.z));float az=1.0-smoothstep(0.0,fwidth(vWorld.x)+uLineWidth*1.5,abs(vWorld.x));vec3 base=vec3(.026,.032,.043)*(0.35+uDay*.65);vec3 color=mix(base,vec3(.12,.14,.17),minor*.38);color=mix(color,vec3(.23,.26,.30),major*.62);color=mix(color,vec3(.65,.11,.08),ax*.8);color=mix(color,vec3(.09,.27,.65),az*.8);color+=spotLighting()*1.5;float radial=1.0-smoothstep(uFadeRadius*.55,uFadeRadius,length(vWorld.xz-uCamera.xz));if(uFogEnabled){float d=distance(uCamera,vWorld);float f=clamp(exp(-pow(uFogDensity*d,2.0)),0.0,1.0);color=mix(uFogColor,color,f);radial*=f;}fragColor=vec4(color,radial);}`;

function compileShader(type, source){
  const shader=gl.createShader(type);gl.shaderSource(shader,source);gl.compileShader(shader);
  if(!gl.getShaderParameter(shader,gl.COMPILE_STATUS))throw new Error(gl.getShaderInfoLog(shader));return shader;
}
function createProgram(vs,fs){const p=gl.createProgram();gl.attachShader(p,compileShader(gl.VERTEX_SHADER,vs));gl.attachShader(p,compileShader(gl.FRAGMENT_SHADER,fs));gl.linkProgram(p);if(!gl.getProgramParameter(p,gl.LINK_STATUS))throw new Error(gl.getProgramInfoLog(p));return p;}

class Mesh {
  constructor(vertices,indices){
    this.vao=gl.createVertexArray();this.count=indices.length;gl.bindVertexArray(this.vao);
    const vb=gl.createBuffer();gl.bindBuffer(gl.ARRAY_BUFFER,vb);gl.bufferData(gl.ARRAY_BUFFER,new Float32Array(vertices),gl.STATIC_DRAW);
    const ib=gl.createBuffer();gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER,ib);gl.bufferData(gl.ELEMENT_ARRAY_BUFFER,new Uint32Array(indices),gl.STATIC_DRAW);
    const stride=14*4;gl.enableVertexAttribArray(0);gl.vertexAttribPointer(0,3,gl.FLOAT,false,stride,0);gl.enableVertexAttribArray(1);gl.vertexAttribPointer(1,3,gl.FLOAT,false,stride,3*4);gl.enableVertexAttribArray(2);gl.vertexAttribPointer(2,2,gl.FLOAT,false,stride,6*4);gl.enableVertexAttribArray(3);gl.vertexAttribPointer(3,3,gl.FLOAT,false,stride,8*4);gl.enableVertexAttribArray(4);gl.vertexAttribPointer(4,3,gl.FLOAT,false,stride,11*4);gl.bindVertexArray(null);
  }
  draw(){gl.bindVertexArray(this.vao);gl.drawElements(gl.TRIANGLES,this.count,gl.UNSIGNED_INT,0);}
}

function cube(half=1){
  const faces=[[[0,0,1],[1,0,0],[0,1,0]],[[0,0,-1],[-1,0,0],[0,1,0]],[[1,0,0],[0,0,-1],[0,1,0]],[[-1,0,0],[0,0,1],[0,1,0]],[[0,1,0],[1,0,0],[0,0,-1]],[[0,-1,0],[1,0,0],[0,0,1]]],v=[],i=[];
  const corners=[[-1,-1],[1,-1],[1,1],[-1,1]],uv=[[0,0],[1,0],[1,1],[0,1]];
  faces.forEach(([n,t,b],f)=>{corners.forEach((c,k)=>v.push((n[0]+t[0]*c[0]+b[0]*c[1])*half,(n[1]+t[1]*c[0]+b[1]*c[1])*half,(n[2]+t[2]*c[0]+b[2]*c[1])*half,...n,...uv[k],...t,...b));const q=f*4;i.push(q,q+1,q+2,q,q+2,q+3);});return new Mesh(v,i);
}
function sphere(radius=1,stacks=28,sectors=36){const v=[],ind=[];for(let y=0;y<=stacks;y++){const phi=Math.PI/2-y*Math.PI/stacks,ring=Math.cos(phi),ny=Math.sin(phi);for(let x=0;x<=sectors;x++){const a=x*2*Math.PI/sectors,nx=ring*Math.cos(a),nz=ring*Math.sin(a),t=[-Math.sin(a),0,Math.cos(a)],b=V3.norm(V3.cross(t,[nx,ny,nz]));v.push(nx*radius,ny*radius,nz*radius,nx,ny,nz,x/sectors,y/stacks,...t,...b);}}for(let y=0;y<stacks;y++)for(let x=0;x<sectors;x++){const a=y*(sectors+1)+x,b=a+sectors+1;ind.push(a,a+1,b,a+1,b+1,b);}return new Mesh(v,ind);}
function torus(major=1.5,minor=.5,majSeg=44,minSeg=20){const v=[],ind=[];for(let a=0;a<=majSeg;a++){const u=a*2*Math.PI/majSeg,cu=Math.cos(u),su=Math.sin(u);for(let b=0;b<=minSeg;b++){const w=b*2*Math.PI/minSeg,cv=Math.cos(w),sv=Math.sin(w),nx=cv*cu,ny=sv,nz=cv*su,t=[-su,0,cu],bt=V3.norm(V3.cross([nx,ny,nz],t));v.push((major+minor*cv)*cu,minor*sv,(major+minor*cv)*su,nx,ny,nz,a/majSeg,b/minSeg,...t,...bt);}}for(let a=0;a<majSeg;a++)for(let b=0;b<minSeg;b++){const x=a*(minSeg+1)+b,y=x+minSeg+1;ind.push(x,x+1,y,x+1,y+1,y);}return new Mesh(v,ind);}
function plane(size=60){const t=[1,0,0],b=[0,0,1];return new Mesh([-size,0,-size,0,1,0,0,0,...t,...b,size,0,-size,0,1,0,1,0,...t,...b,size,0,size,0,1,0,1,1,...t,...b,-size,0,size,0,1,0,0,1,...t,...b],[0,2,1,0,3,2]);}
function cone(segments=32){const v=[0,0,0,0,0,1,.5,0,1,0,0,0,1,0],ind=[];for(let i=0;i<=segments;i++){const a=i*Math.PI*2/segments,c=Math.cos(a),s=Math.sin(a);v.push(c,s,-1,c*.707,s*.707,.707,i/segments,1,-s,c,0,0,0,-1);}for(let i=0;i<segments;i++)ind.push(0,i+1,i+2);const center=v.length/14;v.push(0,0,-1,0,0,-1,.5,1,1,0,0,0,1,0);for(let i=0;i<segments;i++)ind.push(center,i+2,i+1);return new Mesh(v,ind);}

const objectProgram=createProgram(vertexSource,fragmentSource),gridProgram=createProgram(gridVertexSource,gridFragmentSource);
const loc=(p,n)=>gl.getUniformLocation(p,n);
const U={};['uModel','uViewProjection','uNormalMatrix','uCamera','uBaseColor','uSpecular','uShininess','uAmbient','uPhong','uFogEnabled','uFogColor','uFogDensity','uUseNormalMap','uNormalMap','uEmissive','uEmissiveColor','uAlpha','uCone','uLightCount','uLightType[0]','uLightPosition[0]','uLightDirection[0]','uLightColor[0]','uLightIntensity[0]','uInnerCutoff[0]','uOuterCutoff[0]'].forEach(n=>U[n]=loc(objectProgram,n));
const GU={};['uViewProjection','uCamera','uFogColor','uFogDensity','uFogEnabled','uDay','uGridSize','uSubGridSize','uFadeRadius','uLineWidth','uNumSpots','uSpotPosition[0]','uSpotDirection[0]','uSpotColor[0]','uSpotInner[0]','uSpotOuter[0]'].forEach(n=>GU[n]=loc(gridProgram,n));

const meshes={cube:cube(),sphere:sphere(),torus:torus(),marker:sphere(1,12,16),cone:cone(),plane:plane()};
const objects=[
  {name:'Red sphere',mesh:meshes.sphere,pos:[7,1,5],rot:[0,0,0],scale:[1,1,1],color:[.8,.09,.075],spec:[1,.9,.85],shine:64,normal:'brick'},
  {name:'Blue torus',mesh:meshes.torus,pos:[-7,1.5,5],rot:[.3,.5,0],scale:[1,1,1],color:[.08,.2,.9],spec:[.65,.8,1],shine:48},
  {name:'Green cube',mesh:meshes.cube,pos:[-8,.82,-4],rot:[0,.8,0],scale:[.8,.8,.8],color:[.07,.62,.24],spec:[.45,.65,.5],shine:16},
  {name:'Yellow cube',mesh:meshes.cube,pos:[8,.82,-4],rot:[0,-.5,0],scale:[.8,.8,.8],color:[.92,.58,.06],spec:[.8,.7,.35],shine:24},
  {name:'Big sphere',mesh:meshes.sphere,pos:[0,2,-10],rot:[0,0,0],scale:[2,2,2],color:[.38,.42,.5],spec:[.85,.9,1],shine:96,normal:'stone'}
];
const mover={name:'Mover',mesh:meshes.cube,pos:[4,.5,0],rot:[0,0,0],scale:[.5,.5,.5],color:[.9,.6,.1],spec:[1,1,1],shine:32};

const state={camera:0,phong:true,fog:true,fogDensity:.02,day:.8,daySpeed:.15,autoDay:true,normalMaps:false,grid:true,gridSize:1,gridSub:.25,gridFade:60,gridLine:.02,manual:false,driveSpeed:5,turnSpeed:2,point:true,spot:true,sun:true,head:true,pointIntensity:1,spotIntensity:1,sunIntensity:1,headIntensity:1,aimYaw:0,aimPitch:0,freeSpeed:8,freeSensitivity:.3,keys:new Set(),orbit:0,orbitRadius:4,moverRotation:0,driveHeading:0};
const camera={yaw:.78,pitch:.45,distance:22,target:[0,1,0],eye:[12,10,12],freeYaw:-2.35,freePitch:-.42,freeEye:[12,10,12],drag:false,lastX:0,lastY:0};
const cameraNames=[['STATIC','Overview camera'],['TRACKING','Follows the mover'],['THIRD PERSON','Chase camera'],['FIRST PERSON','Driver viewpoint'],['FREE','WASD + Q/E to move']];
let lastCameraPose={eye:[12,10,12],target:[0,0,0]};

function createTexture(url){
  const texture=gl.createTexture();texture.loaded=false;gl.bindTexture(gl.TEXTURE_2D,texture);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,1,1,0,gl.RGBA,gl.UNSIGNED_BYTE,new Uint8Array([128,128,255,255]));gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_S,gl.REPEAT);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_WRAP_T,gl.REPEAT);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MAG_FILTER,gl.LINEAR);
  const image=new Image();image.onload=()=>{gl.bindTexture(gl.TEXTURE_2D,texture);gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL,true);gl.pixelStorei(gl.UNPACK_COLORSPACE_CONVERSION_WEBGL,gl.NONE);gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL,false);gl.texImage2D(gl.TEXTURE_2D,0,gl.RGBA,gl.RGBA,gl.UNSIGNED_BYTE,image);gl.generateMipmap(gl.TEXTURE_2D);gl.texParameteri(gl.TEXTURE_2D,gl.TEXTURE_MIN_FILTER,gl.LINEAR_MIPMAP_LINEAR);texture.loaded=true;};image.onerror=()=>console.error(`Could not load normal map: ${url}`);image.src=url;return texture;
}
const textures={brick:createTexture('./assets/brick_normalmap.png'),stone:createTexture('./assets/normal_map.jpg')};

function setupObjectUniforms(viewProjection,eye,fogColor,lights){
  gl.useProgram(objectProgram);gl.uniformMatrix4fv(U.uViewProjection,false,viewProjection);gl.uniform3fv(U.uCamera,eye);gl.uniform1f(U.uAmbient,.15+.85*state.day);gl.uniform1i(U.uPhong,state.phong);gl.uniform1i(U.uFogEnabled,state.fog);gl.uniform3fv(U.uFogColor,fogColor);gl.uniform1f(U.uFogDensity,state.fogDensity);gl.uniform1i(U.uNormalMap,0);gl.uniform1i(U.uLightCount,lights.length);
  const types=new Int32Array(5),positions=new Float32Array(15),directions=new Float32Array(15),colors=new Float32Array(15),intensities=new Float32Array(5),inner=new Float32Array(5),outer=new Float32Array(5);
  lights.forEach((l,i)=>{types[i]=l.type;positions.set(l.position||[0,0,0],i*3);directions.set(l.direction||[0,-1,0],i*3);colors.set(l.color,i*3);intensities[i]=l.intensity;inner[i]=Math.cos((l.inner||15)*Math.PI/180);outer[i]=Math.cos((l.outer||25)*Math.PI/180);});
  gl.uniform1iv(U['uLightType[0]'],types);gl.uniform3fv(U['uLightPosition[0]'],positions);gl.uniform3fv(U['uLightDirection[0]'],directions);gl.uniform3fv(U['uLightColor[0]'],colors);gl.uniform1fv(U['uLightIntensity[0]'],intensities);gl.uniform1fv(U['uInnerCutoff[0]'],inner);gl.uniform1fv(U['uOuterCutoff[0]'],outer);
}

function drawObject(obj,{emissive=null,alpha=1,cone=false}={}){
  const model=obj.model||M4.compose(obj.pos,obj.rot,obj.scale);gl.uniformMatrix4fv(U.uModel,false,model);gl.uniformMatrix3fv(U.uNormalMatrix,false,M4.normal3(model));gl.uniform3fv(U.uBaseColor,obj.color);gl.uniform3fv(U.uSpecular,obj.spec||[.7,.7,.7]);gl.uniform1f(U.uShininess,obj.shine||32);gl.uniform1i(U.uEmissive,!!emissive);gl.uniform3fv(U.uEmissiveColor,emissive||[0,0,0]);gl.uniform1f(U.uAlpha,alpha);gl.uniform1i(U.uCone,cone);
  const useMap=state.normalMaps&&obj.normal&&textures[obj.normal];gl.uniform1i(U.uUseNormalMap,!!useMap);if(useMap){gl.activeTexture(gl.TEXTURE0);gl.bindTexture(gl.TEXTURE_2D,textures[obj.normal]);}obj.mesh.draw();
}

function currentCamera(dt){
  const p=mover.pos,yaw=mover.rot[1];
  if(state.camera===0){const cp=Math.cos(camera.pitch);camera.eye=[camera.target[0]+camera.distance*cp*Math.sin(camera.yaw),camera.target[1]+camera.distance*Math.sin(camera.pitch),camera.target[2]+camera.distance*cp*Math.cos(camera.yaw)];return {eye:camera.eye,target:camera.target};}
  if(state.camera===1)return {eye:[0,12,-14],target:p};
  if(state.camera===2){const forward=[Math.sin(yaw),0,Math.cos(yaw)];return {eye:V3.add(V3.sub(p,V3.scale(forward,5)),[0,5,0]),target:V3.add(p,[0,1,0])};}
  if(state.camera===3){const forward=[Math.sin(yaw),0,Math.cos(yaw)];const eye=V3.add(p,V3.add(V3.scale(forward,.55),[0,.2,0]));return {eye,target:V3.add(eye,V3.scale(forward,10))};}
  const arrowYaw=(state.keys.has('ArrowRight')?1:0)-(state.keys.has('ArrowLeft')?1:0),arrowPitch=(state.keys.has('ArrowUp')?1:0)-(state.keys.has('ArrowDown')?1:0);camera.freeYaw+=arrowYaw*1.15*dt;camera.freePitch=Math.max(-1.553,Math.min(1.553,camera.freePitch+arrowPitch*1.15*dt));
  const f=[Math.cos(camera.freePitch)*Math.sin(camera.freeYaw),Math.sin(camera.freePitch),Math.cos(camera.freePitch)*Math.cos(camera.freeYaw)],r=[Math.cos(camera.freeYaw),0,-Math.sin(camera.freeYaw)];let move=[0,0,0];if(state.keys.has('KeyW'))move=V3.add(move,f);if(state.keys.has('KeyS'))move=V3.sub(move,f);if(state.keys.has('KeyD'))move=V3.add(move,r);if(state.keys.has('KeyA'))move=V3.sub(move,r);if(state.keys.has('KeyE'))move[1]++;if(state.keys.has('KeyQ'))move[1]--;if(V3.length(move))camera.freeEye=V3.add(camera.freeEye,V3.scale(V3.norm(move),state.freeSpeed*dt));return {eye:camera.freeEye,target:V3.add(camera.freeEye,f)};
}

function updateMover(dt){
  if(state.manual){if(state.camera!==4){let drive=(state.keys.has('KeyW')?1:0)-(state.keys.has('KeyS')?1:0),turn=(state.keys.has('KeyD')?1:0)-(state.keys.has('KeyA')?1:0);if(drive<0)turn=-turn;state.driveHeading+=turn*state.turnSpeed*dt;mover.rot[1]=state.driveHeading;mover.pos[0]+=Math.sin(state.driveHeading)*drive*state.driveSpeed*dt;mover.pos[2]+=Math.cos(state.driveHeading)*drive*state.driveSpeed*dt;}}
  else{state.orbit+=.4*dt;state.moverRotation+=1.5*dt;mover.pos[0]=Math.cos(state.orbit)*state.orbitRadius;mover.pos[2]=Math.sin(state.orbit)*state.orbitRadius;mover.rot[1]=state.moverRotation;}
  if(state.camera!==4){const yaw=(state.keys.has('ArrowLeft')?1:0)-(state.keys.has('ArrowRight')?1:0),pitch=(state.keys.has('ArrowUp')?1:0)-(state.keys.has('ArrowDown')?1:0);if(yaw||pitch){state.aimYaw=Math.max(-1,Math.min(1,state.aimYaw+yaw*1.2*dt));state.aimPitch=Math.max(-1,Math.min(1,state.aimPitch+pitch*1.2*dt));syncAimControls();}}
}

function getLights(){
  const yaw=mover.rot[1],forward=[Math.sin(yaw),0,Math.cos(yaw)],right=[Math.cos(yaw),0,-Math.sin(yaw)];const aim=V3.norm(V3.add(V3.add(V3.scale(right,Math.sin(state.aimYaw*.5)),V3.scale(forward,Math.cos(state.aimYaw*.5))),[0,-.3+state.aimPitch*.5,0]));
  const list=[];if(state.point)list.push({type:0,position:[0,10,0],color:[.9,.85,.7],intensity:state.pointIntensity});if(state.spot)list.push({type:1,position:[-7,7,5],direction:[0,-1,0],color:[.4,.4,.8],intensity:state.spotIntensity,inner:20,outer:35});if(state.sun)list.push({type:2,direction:[-.3,-1,-.5],color:[1,.95,.85],intensity:state.sunIntensity*state.day});
  const headPositions=[];[-.25,.25].forEach(x=>{const pos=V3.add(mover.pos,V3.add(V3.scale(right,x),V3.add(V3.scale(forward,.5),[0,.15,0])));headPositions.push(pos);if(state.head)list.push({type:1,position:pos,direction:aim,color:[1,1,.9],intensity:state.headIntensity,inner:15,outer:25});});return {list,headPositions,aim};
}

let carParts=null,carLoading=false;
async function loadCar(){
  if(carParts||carLoading)return;carLoading=true;const status=document.querySelector('#car-status');status.textContent='Downloading model and materials…';
  try{const [objResponse,mtlResponse]=await Promise.all([fetch('./assets/models/car/sportsCar.obj'),fetch('./assets/models/car/sportsCar.mtl')]);if(!objResponse.ok)throw new Error(objResponse.statusText);if(!mtlResponse.ok)throw new Error(mtlResponse.statusText);const [objText,mtlText]=await Promise.all([objResponse.text(),mtlResponse.text()]);status.textContent='Building material meshes…';await new Promise(r=>requestAnimationFrame(r));const materials=parseMtl(mtlText);carParts=parseObj(objText,materials);status.textContent=`Loaded with ${carParts.length} materials`;}
  catch(e){console.error(e);status.textContent='Could not load model';document.querySelector('#show-car').checked=false;}finally{carLoading=false;}
}
function parseMtl(text){
  const materials=new Map();let current=null;for(const line of text.split(/\r?\n/)){const s=line.trim().split(/\s+/),command=s[0];if(command==='newmtl'){current={color:[.5,.5,.5],spec:[.6,.6,.6],shine:32};materials.set(s.slice(1).join(' '),current);}else if(current&&command==='Kd')current.color=s.slice(1,4).map(Number);else if(current&&command==='Ks')current.spec=s.slice(1,4).map(Number);else if(current&&command==='Ns')current.shine=Math.max(1,Number(s[1])||32);}return materials;
}
function parseObj(text,materials){
  const positions=[],normals=[],uvs=[],groups=new Map();let materialName='__default';
  const group=()=>{if(!groups.has(materialName))groups.set(materialName,{vertices:[],indices:[],cache:new Map()});return groups.get(materialName);};
  for(const line of text.split(/\r?\n/)){const s=line.trim().split(/\s+/),command=s[0];if(command==='v')positions.push(s.slice(1,4).map(Number));else if(command==='vn')normals.push(s.slice(1,4).map(Number));else if(command==='vt')uvs.push(s.slice(1,3).map(Number));else if(command==='usemtl')materialName=s.slice(1).join(' ');else if(command==='f'){const g=group(),face=s.slice(1).map(key=>{if(g.cache.has(key))return g.cache.get(key);const q=key.split('/').map(Number),pos=positions[q[0]-1]||[0,0,0],uv=uvs[q[1]-1]||[0,0],normal=normals[q[2]-1]||[0,1,0],index=g.vertices.length/14;g.vertices.push(...pos,...normal,uv[0],uv[1],0,0,0,0,0,0);g.cache.set(key,index);return index;});for(let i=1;i<face.length-1;i++)g.indices.push(face[0],face[i],face[i+1]);}}
  const fallback={color:[.5,.5,.5],spec:[.6,.6,.6],shine:32};return [...groups].map(([name,g])=>({mesh:new Mesh(g.vertices,g.indices),...(materials.get(name)||fallback)}));
}

function resize(){const dpr=Math.min(devicePixelRatio||1,2),w=Math.floor(canvas.clientWidth*dpr),h=Math.floor(canvas.clientHeight*dpr);if(canvas.width!==w||canvas.height!==h){canvas.width=w;canvas.height=h;}gl.viewport(0,0,w,h);return w/Math.max(h,1);}
function projectTag(id,position,vp,visible=true){const el=document.querySelector(id);if(!visible){el.style.opacity=0;return;}const clip=[vp[0]*position[0]+vp[4]*position[1]+vp[8]*position[2]+vp[12],vp[1]*position[0]+vp[5]*position[1]+vp[9]*position[2]+vp[13],vp[3]*position[0]+vp[7]*position[1]+vp[11]*position[2]+vp[15]];if(clip[2]<=0){el.style.opacity=0;return;}el.style.opacity=1;el.style.left=`${(clip[0]/clip[2]*.5+.5)*innerWidth}px`;el.style.top=`${(-clip[1]/clip[2]*.5+.5)*innerHeight}px`;}

let last=performance.now(),fpsSmooth=60,statTimer=0,dayDirection=1;
function frame(now){
  const dt=Math.min((now-last)/1000,.05);last=now;fpsSmooth=fpsSmooth*.92+(1/Math.max(dt,.001))*.08;updateMover(dt);if(state.autoDay){state.day+=dayDirection*state.daySpeed*.5*dt;if(state.day>=1||state.day<=0){dayDirection*=-1;state.day=Math.max(0,Math.min(1,state.day));}syncDayControl();}
  const aspect=resize(),cam=currentCamera(dt),view=M4.lookAt(cam.eye,cam.target),projection=M4.perspective(55*Math.PI/180,aspect,.1,200),vp=M4.multiply(projection,view),lights=getLights();lastCameraPose={eye:[...cam.eye],target:[...cam.target]};const fogColor=[.018+.16*state.day,.025+.25*state.day,.055+.42*state.day];
  gl.enable(gl.DEPTH_TEST);gl.enable(gl.CULL_FACE);gl.cullFace(gl.BACK);gl.clearColor(fogColor[0],fogColor[1],fogColor[2],1);gl.clear(gl.COLOR_BUFFER_BIT|gl.DEPTH_BUFFER_BIT);
  if(state.grid){
    gl.useProgram(gridProgram);gl.uniformMatrix4fv(GU.uViewProjection,false,vp);gl.uniform3fv(GU.uCamera,cam.eye);gl.uniform3fv(GU.uFogColor,fogColor);gl.uniform1f(GU.uFogDensity,state.fogDensity);gl.uniform1i(GU.uFogEnabled,state.fog);gl.uniform1f(GU.uDay,state.day);gl.uniform1f(GU.uGridSize,state.gridSize);gl.uniform1f(GU.uSubGridSize,state.gridSub);gl.uniform1f(GU.uFadeRadius,state.gridFade);gl.uniform1f(GU.uLineWidth,state.gridLine);
    const spots=lights.list.filter(light=>light.type===1).slice(0,4),positions=new Float32Array(12),directions=new Float32Array(12),colors=new Float32Array(12),inner=new Float32Array(4),outer=new Float32Array(4);spots.forEach((light,i)=>{positions.set(light.position,i*3);directions.set(light.direction,i*3);colors.set(light.color.map(c=>c*light.intensity),i*3);inner[i]=Math.cos(light.inner*Math.PI/180);outer[i]=Math.cos(light.outer*Math.PI/180);});gl.uniform1i(GU.uNumSpots,spots.length);gl.uniform3fv(GU['uSpotPosition[0]'],positions);gl.uniform3fv(GU['uSpotDirection[0]'],directions);gl.uniform3fv(GU['uSpotColor[0]'],colors);gl.uniform1fv(GU['uSpotInner[0]'],inner);gl.uniform1fv(GU['uSpotOuter[0]'],outer);
    gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);meshes.plane.draw();gl.disable(gl.BLEND);
  }
  setupObjectUniforms(vp,cam.eye,fogColor,lights.list);objects.forEach(o=>drawObject(o));drawObject(mover);
  if(document.querySelector('#show-car').checked&&carParts)carParts.forEach(part=>drawObject({...part,pos:[12,0,0],rot:[0,-Math.PI/2,0],scale:[2,2,2]}));
  lights.list.filter(light=>light.type!==2).forEach(light=>drawObject({mesh:meshes.marker,pos:light.position,rot:[0,0,0],scale:[.2,.2,.2],color:[1,1,1]}, {emissive:light.color.map(c=>c*Math.max(.35,light.intensity))}));
  gl.enable(gl.BLEND);gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);gl.depthMask(false);gl.disable(gl.CULL_FACE);lights.list.filter(light=>light.type===1).forEach(light=>{const range=15,radius=range*Math.tan(light.outer*Math.PI/180)*.5,model=M4.multiply(M4.translation(light.position),M4.multiply(M4.lookRotation(light.direction),M4.scaling([radius,radius,range])));drawObject({mesh:meshes.cone,model,color:[1,1,1]}, {emissive:light.color.map(c=>c*light.intensity*.72),alpha:.22,cone:true});});gl.depthMask(true);gl.enable(gl.CULL_FACE);gl.disable(gl.BLEND);
  projectTag('#label-key',[0,10.45,0],vp,state.point);projectTag('#label-spot',[-7,7.4,5],vp,state.spot);projectTag('#label-mover',V3.add(mover.pos,[0,1.2,0]),vp,true);
  statTimer+=dt;if(statTimer>.25){document.querySelector('#fps').textContent=Math.round(fpsSmooth);document.querySelector('#frame-time').innerHTML=`${(1000/fpsSmooth).toFixed(1)} <small>ms</small>`;statTimer=0;}requestAnimationFrame(frame);
}

function byId(id){return document.getElementById(id)}
function bindCheck(id,key,after){const el=byId(id);el.addEventListener('change',()=>{state[key]=el.checked;after?.(el.checked);});}
function bindRange(id,key,out,format=v=>v){const el=byId(id);el.addEventListener('input',()=>{state[key]=Number(el.value);if(out)byId(out).textContent=format(el.value);});}
function setCamera(value){const next=Number(value);if(next===4&&state.camera!==4){const forward=V3.norm(V3.sub(lastCameraPose.target,lastCameraPose.eye));camera.freeEye=[...lastCameraPose.eye];camera.freeYaw=Math.atan2(forward[0],forward[2]);camera.freePitch=Math.asin(Math.max(-1,Math.min(1,forward[1])));}state.camera=next;document.querySelectorAll('#camera-control button').forEach(b=>b.classList.toggle('active',Number(b.dataset.value)===state.camera));const [name,desc]=cameraNames[state.camera];byId('camera-label').innerHTML=`<span>${name}</span>${desc}`;byId('camera-hud').textContent=`${name} CAMERA`;byId('control-hint').textContent=state.camera===4?'WASD + Q/E · hold RMB and drag to look':state.camera===0?'Drag to orbit · scroll to zoom':'Camera follows the animated object';byId('free-camera-options').hidden=state.camera!==4;}
function syncDayControl(){byId('day-factor').value=state.day;byId('day-value').textContent=`${Math.round(state.day*100)}%`;}
function syncAimControls(){byId('aim-yaw').value=state.aimYaw;byId('aim-pitch').value=state.aimPitch;}
document.querySelectorAll('.section-title').forEach(button=>button.addEventListener('click',()=>{const s=button.parentElement;s.classList.toggle('open');button.querySelector('b').textContent=s.classList.contains('open')?'−':'+';}));
document.querySelectorAll('#camera-control button').forEach(b=>b.addEventListener('click',()=>setCamera(b.dataset.value)));
document.querySelectorAll('#shading-control button').forEach(b=>b.addEventListener('click',()=>{state.phong=b.dataset.value==='phong';document.querySelectorAll('#shading-control button').forEach(x=>x.classList.toggle('active',x===b));}));
bindRange('day-factor','day','day-value',v=>`${Math.round(v*100)}%`);bindRange('day-speed','daySpeed','day-speed-value',v=>Number(v).toFixed(2));bindRange('fog-density','fogDensity','fog-value',v=>Number(v).toFixed(3));bindRange('free-speed','freeSpeed','free-speed-value',v=>Number(v).toFixed(1));bindRange('free-sensitivity','freeSensitivity','free-sensitivity-value',v=>Number(v).toFixed(2));bindRange('drive-speed','driveSpeed','drive-speed-value',v=>Number(v).toFixed(1));bindRange('turn-speed','turnSpeed','turn-speed-value',v=>Number(v).toFixed(1));bindRange('grid-size','gridSize','grid-size-value',v=>Number(v).toFixed(2));bindRange('grid-sub','gridSub','grid-sub-value',v=>Number(v).toFixed(2));bindRange('grid-fade','gridFade','grid-fade-value',v=>Math.round(v));bindRange('grid-line','gridLine','grid-line-value',v=>Number(v).toFixed(3));bindRange('point-intensity','pointIntensity');bindRange('spot-intensity','spotIntensity');bindRange('sun-intensity','sunIntensity');bindRange('head-intensity','headIntensity');bindRange('aim-yaw','aimYaw');bindRange('aim-pitch','aimPitch');
bindCheck('day-auto','autoDay');bindCheck('fog-enabled','fog');bindCheck('normal-maps','normalMaps');bindCheck('grid-visible','grid');bindCheck('manual-drive','manual',checked=>{if(checked){state.driveHeading=mover.rot[1];}else{state.orbit=Math.atan2(mover.pos[2],mover.pos[0]);state.orbitRadius=Math.max(.1,Math.hypot(mover.pos[0],mover.pos[2]));state.moverRotation=mover.rot[1];}byId('pause-button').classList.toggle('active',checked);});bindCheck('point-enabled','point');bindCheck('spot-enabled','spot');bindCheck('sun-enabled','sun');bindCheck('head-enabled','head');bindCheck('show-car','showCar',checked=>{if(checked)loadCar();});
byId('reset-aim').addEventListener('click',()=>{state.aimYaw=0;state.aimPitch=0;syncAimControls();});

const panel=byId('panel');const togglePanel=()=>panel.classList.toggle('hidden');byId('panel-button').addEventListener('click',togglePanel);byId('close-panel').addEventListener('click',togglePanel);
function toggleDrive(){byId('manual-drive').click();}byId('pause-button').addEventListener('click',toggleDrive);
window.addEventListener('keydown',e=>{if(e.target.matches('input,button'))return;state.keys.add(e.code);if(e.code.startsWith('Arrow'))e.preventDefault();if(e.repeat)return;if(/^Digit[1-5]$/.test(e.code))setCamera(Number(e.code.slice(-1))-1);if(e.code==='KeyP')document.querySelector(`#shading-control button[data-value="${state.phong?'gouraud':'phong'}"]`).click();if(e.code==='KeyF')byId('fog-enabled').click();if(e.code==='KeyN')byId('day-auto').click();if(e.code==='KeyO'){state.day=Math.min(1,state.day+.05);syncDayControl();}if(e.code==='KeyL'){state.day=Math.max(0,state.day-.05);syncDayControl();}if(e.code==='Equal'||e.code==='NumpadAdd'){state.fogDensity=Math.min(.1,state.fogDensity+.005);byId('fog-density').value=state.fogDensity;byId('fog-value').textContent=state.fogDensity.toFixed(3);}if(e.code==='Minus'||e.code==='NumpadSubtract'){state.fogDensity=Math.max(0,state.fogDensity-.005);byId('fog-density').value=state.fogDensity;byId('fog-value').textContent=state.fogDensity.toFixed(3);}if(e.code==='Tab'){e.preventDefault();togglePanel();}if(e.code==='Space'){e.preventDefault();toggleDrive();}});window.addEventListener('keyup',e=>state.keys.delete(e.code));window.addEventListener('blur',()=>state.keys.clear());
canvas.addEventListener('contextmenu',e=>e.preventDefault());canvas.addEventListener('pointerdown',e=>{if(state.camera===4&&e.pointerType==='mouse'&&e.button!==2)return;camera.drag=true;camera.lastX=e.clientX;camera.lastY=e.clientY;canvas.setPointerCapture(e.pointerId);});canvas.addEventListener('pointerup',()=>camera.drag=false);canvas.addEventListener('pointercancel',()=>camera.drag=false);canvas.addEventListener('pointermove',e=>{if(!camera.drag)return;const dx=e.clientX-camera.lastX,dy=e.clientY-camera.lastY;camera.lastX=e.clientX;camera.lastY=e.clientY;if(state.camera===4){const sensitivity=state.freeSensitivity*.01;camera.freeYaw+=dx*sensitivity;camera.freePitch=Math.max(-1.553,Math.min(1.553,camera.freePitch-dy*sensitivity));}else if(state.camera===0){camera.yaw-=dx*.006;camera.pitch=Math.max(.08,Math.min(1.45,camera.pitch+dy*.006));}});canvas.addEventListener('wheel',e=>{if(state.camera===0){e.preventDefault();camera.distance=Math.max(7,Math.min(55,camera.distance+e.deltaY*.018));}},{passive:false});

requestAnimationFrame(frame);requestAnimationFrame(()=>setTimeout(()=>byId('loading').classList.add('done'),250));
