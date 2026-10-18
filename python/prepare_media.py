"""Create portable original audio, photo portrait, and fetch an OFL Korean font."""
from pathlib import Path
import math
import urllib.request
import wave
import numpy as np
from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / 'assets'
for d in ['audio','fonts','images']:(ASSETS/d).mkdir(parents=True,exist_ok=True)
font_base='https://raw.githubusercontent.com/orioncactus/pretendard/v1.3.9/'
for name,url in [('Pretendard-Regular.otf',font_base+'packages/pretendard/dist/public/static/Pretendard-Regular.otf'),('Pretendard-SemiBold.otf',font_base+'packages/pretendard/dist/public/static/Pretendard-SemiBold.otf'),('Pretendard-OFL.txt',font_base+'LICENSE'),('FiraMono-OFL.txt','https://raw.githubusercontent.com/google/fonts/main/ofl/firamono/OFL.txt')]:
    dest=ASSETS/'fonts'/name
    if not dest.exists():urllib.request.urlretrieve(url,dest)
photo=Image.open(ROOT/'reference/cat/20231121_000119_crop.jpg').convert('RGB')
ImageOps.fit(photo,(512,512)).save(ASSETS/'images/cat_portrait.png')

RATE=22050
def write(name,samples):
    samples=np.asarray(samples)
    if samples.ndim==1:samples=np.column_stack((samples,samples))
    pcm=(np.clip(samples,-1,1)*32767).astype('<i2')
    with wave.open(str(ASSETS/'audio'/name),'wb') as w:
        w.setnchannels(2);w.setsampwidth(2);w.setframerate(RATE);w.writeframes(pcm.tobytes())
def note(freq,duration,amp=.12):
    t=np.arange(int(duration*RATE))/RATE
    env=(1-np.exp(-t*35))*np.exp(-t*2.7)
    return amp*env*(np.sin(t*freq*math.tau)+.22*np.sin(t*freq*2*math.tau)+.08*np.sin(t*freq*3*math.tau))
def hz(m):return 440*2**((m-69)/12)
# Original 32 second, seamlessly looping, slow felt music-box piece in C.
length=32
music=np.zeros((length*RATE,2),dtype=np.float64)
chords=[[48,55,60,64],[45,52,57,60],[53,60,65,69],[43,50,55,59]]
melody=[76,79,81,79,76,74,72,67,69,72,76,74,72,71,67,64]
def add_at(buf,at,s,pan=0):
    start=int(at*RATE)
    for ch,gain in enumerate([.7-pan*.25,.7+pan*.25]):
        for off in [0,len(buf)]:
            lo=max(0,start-off);src=max(0,off-start);n=min(len(s)-src,len(buf)-lo)
            if n>0:buf[lo:lo+n,ch]+=s[src:src+n]*gain
for bar in range(8):
    chord=chords[bar%4]
    for k,m in enumerate(chord):add_at(music,bar*4+k*.5,note(hz(m),3.3,.10),(-1)**k*.4)
    for k in range(2):add_at(music,bar*4+k*2,note(hz(melody[bar*2+k]),3,.075),.25)
write('evening.wav',music)
for name,notes in [('discover.wav',[76,79,84]),('deduce.wav',[72,76,79,84]),('click.wav',[81]),('reunion.wav',[72,76,79,84,88])]:
    a=np.zeros(int((len(notes)*.13+1.5)*RATE))
    for i,m in enumerate(notes):
        s=note(hz(m),1.4,.18);start=int(i*.13*RATE);a[start:start+len(s)]+=s
    write(name,a)
t=np.arange(int(.65*RATE))/RATE
freq=550+150*np.sin(t*math.pi/.65)
meow=.09*np.sin(math.tau*np.cumsum(freq)/RATE)*np.sin(math.pi*t/.65)**2
write('meow.wav',meow)
print('Prepared Korean font, photo portrait, and 6 original audio files.')
