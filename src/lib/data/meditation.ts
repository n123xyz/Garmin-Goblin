export interface MeditationStep {
  title: string;
  instruction: string;
  durationSec: number;
  spokenGuidance: string;
  targetZone?: string; // For body scan (e.g. 'right-foot', 'torso', 'head')
  senseType?: 'see' | 'feel' | 'hear' | 'smell' | 'taste'; // For 5-4-3-2-1
  count?: number; // 5, 4, 3, 2, 1
}

export interface MeditationExercise {
  id: string;
  title: string;
  subtitle: string;
  bestFor: string;
  defaultDurationMin: number;
  minDurationMin: number;
  maxDurationMin: number;
  accentColor: string;
  icon: string;
  description: string;
  whyItWorks: string;
  steps: MeditationStep[];
}

export const MEDITATION_EXERCISES: MeditationExercise[] = [
  {
    id: 'box-breathing',
    title: 'Box Breathing',
    subtitle: 'For Immediate Calm & Autonomic Balance',
    bestFor: 'Acute stress, anxiety, or preparing for high-pressure focus.',
    defaultDurationMin: 4,
    minDurationMin: 2,
    maxDurationMin: 10,
    accentColor: 'cyan',
    icon: '⏹️',
    description:
      'A powerful 4-4-4-4 tactical breathing practice used by athletes and first responders to rapidly down-regulate the nervous system and regain mental composure.',
    whyItWorks:
      'Even-ratio breath holding stimulates the vagus nerve, immediately slowing heart rate, lowering blood pressure, and activating parasympathetic recovery.',
    steps: [
      {
        title: 'Posture & Clear Lungs',
        instruction: 'Sit upright with your feet flat on the floor. Close your eyes and exhale completely through your mouth.',
        durationSec: 15,
        spokenGuidance:
          'Welcome to Box Breathing. Sit tall with your feet flat on the floor. Soften your shoulders, close your eyes, and exhale all the air completely from your lungs.',
      },
      {
        title: 'Inhale (4s)',
        instruction: 'Inhale deeply through your nose for a count of 4. Feel your belly and lower ribs expand.',
        durationSec: 4,
        spokenGuidance: 'Inhale deeply through your nose. One, two, three, four.',
      },
      {
        title: 'Hold Full (4s)',
        instruction: 'Hold your breath gently with lungs full for a count of 4.',
        durationSec: 4,
        spokenGuidance: 'Hold your breath gently with lungs full. One, two, three, four.',
      },
      {
        title: 'Exhale (4s)',
        instruction: 'Exhale smoothly and completely through your mouth for a count of 4.',
        durationSec: 4,
        spokenGuidance: 'Exhale smoothly through your mouth. One, two, three, four.',
      },
      {
        title: 'Hold Empty (4s)',
        instruction: 'Hold your breath with lungs empty for a count of 4.',
        durationSec: 4,
        spokenGuidance: 'Hold empty and still. One, two, three, four.',
      },
    ],
  },
  {
    id: 'grounding-54321',
    title: 'The 5-4-3-2-1 Technique',
    subtitle: 'Active Sensory Grounding',
    bestFor: 'Stopping racing thoughts, panic attacks, or general overwhelm.',
    defaultDurationMin: 3,
    minDurationMin: 2,
    maxDurationMin: 6,
    accentColor: 'emerald',
    icon: '👁️',
    description:
      'An active, eyes-open sensory mindfulness practice designed to anchor your mind into the present moment by processing immediate tactile and sensory data.',
    whyItWorks:
      'Engages the sensory cortex and redirects cognitive bandwidth away from amygdala-driven thought loops back to immediate physical reality.',
    steps: [
      {
        title: '5 Things You Can See',
        instruction: 'Look around your environment. Notice 5 small visual details you usually ignore (a shadow, a texture, a crack, a leaf).',
        durationSec: 35,
        senseType: 'see',
        count: 5,
        spokenGuidance:
          'Open your eyes and look around your space. Mentally note five things you can see. Look for subtle details you normally overlook, like shadows, textures, or shapes.',
      },
      {
        title: '4 Things You Can Feel',
        instruction: 'Notice 4 physical sensations: the fabric of your shirt, the weight in your chair, the temperature of the air.',
        durationSec: 30,
        senseType: 'feel',
        count: 4,
        spokenGuidance:
          'Now bring attention to your physical body. Identify four things you can physically feel: the contact of your chair, the texture of your clothing, or the air on your skin.',
      },
      {
        title: '3 Things You Can Hear',
        instruction: 'Listen past the obvious sounds. Notice 3 sounds: a distant hum, the breeze, or your own rhythmic breath.',
        durationSec: 30,
        senseType: 'hear',
        count: 3,
        spokenGuidance:
          'Tune your ears outward. Listen for three distinct sounds around you. Look past the loudest noise to find subtle background sounds or your own breathing.',
      },
      {
        title: '2 Things You Can Smell',
        instruction: 'Notice 2 scents in the air, or recall your favorite calming scents like cedar or morning coffee.',
        durationSec: 25,
        senseType: 'smell',
        count: 2,
        spokenGuidance:
          'Take a gentle breath in through your nose. Notice two things you can smell. If the air is neutral, recall two of your favorite comforting aromas.',
      },
      {
        title: '1 Thing You Can Taste',
        instruction: 'Focus on the current taste in your mouth, or take a sip of water and notice the pure physical sensation.',
        durationSec: 20,
        senseType: 'taste',
        count: 1,
        spokenGuidance:
          'Finally, bring your awareness to taste. Notice the current taste in your mouth or the feeling of your tongue resting against the roof of your mouth.',
      },
    ],
  },
  {
    id: 'leaves-on-a-stream',
    title: 'Leaves on a Stream',
    subtitle: 'Detaching from Intrusive Thoughts',
    bestFor: 'Overthinkers, persistent worries, and cognitive defusion.',
    defaultDurationMin: 6,
    minDurationMin: 3,
    maxDurationMin: 12,
    accentColor: 'amber',
    icon: '🍃',
    description:
      'An Acceptance and Commitment Therapy (ACT) meditation that guides you to observe your thoughts objectively as leaves floating past on a gentle river.',
    whyItWorks:
      'Develops cognitive defusion—the ability to recognize thoughts as temporary mental events rather than absolute commands or crises that require panic.',
    steps: [
      {
        title: 'Arrive at the Riverbank',
        instruction: 'Sit comfortably, close your eyes, and imagine you are seated beside a gently flowing stream with leaves floating past.',
        durationSec: 45,
        spokenGuidance:
          'Settle in comfortably and close your eyes. Imagine yourself sitting on the lush bank of a clear, gently flowing stream. Watch the leaves softly drifting past on the surface of the water.',
      },
      {
        title: 'Place Thoughts on Leaves',
        instruction: 'Whenever a thought arises—worries, memories, boredom—gently place it onto a leaf and watch it float downstream.',
        durationSec: 120,
        spokenGuidance:
          'Whenever a thought pops into your mind, whether a worry, a planning thought, or simple restlessness, gently place that thought onto a leaf. Watch it drift smoothly downstream until it fades from view.',
      },
      {
        title: 'Let Stuck Thoughts Float',
        instruction: 'If a leaf gets stuck, do not push it. Simply watch it without judgment until the current carries it away.',
        durationSec: 120,
        spokenGuidance:
          'If a thought lingers or a leaf gets caught near a rock, do not force it away. Simply watch it with calm curiosity. In its own time, the current will carry it away.',
      },
      {
        title: 'Returning to Stillness',
        instruction: 'Acknowledge your capacity to observe thoughts without becoming them. Take a deep centering breath.',
        durationSec: 45,
        spokenGuidance:
          'Take a deep, slow breath. Notice that you are the observer of the river, peaceful and unmoved by whatever floats past. Rest in this stillness.',
      },
    ],
  },
  {
    id: 'body-scan',
    title: 'The Body Scan',
    subtitle: 'Yoga Nidra Deep Somatic Release',
    bestFor: 'Unwinding before bed, releasing deep physical tension, and restful recovery.',
    defaultDurationMin: 10,
    minDurationMin: 5,
    maxDurationMin: 20,
    accentColor: 'purple',
    icon: '✨',
    description:
      'A deep somatic relaxation practice inspired by Yoga Nidra that systematically guides awareness through every muscle group, releasing unconscious stress.',
    whyItWorks:
      'Systematic neuromuscular scanning calms the motor cortex, triggers vasodilation, and helps induce alpha and theta brainwave states for deep restoration.',
    steps: [
      {
        title: 'Settling In',
        instruction: 'Lie flat on your back in a comfortable position. Take 3 deep breaths and feel your body sinking heavier with each exhale.',
        durationSec: 60,
        targetZone: 'full-body',
        spokenGuidance:
          'Lie flat on your back in complete comfort. Close your eyes and take three slow, expansive breaths. With every exhale, feel your entire body sinking heavier into the surface beneath you.',
      },
      {
        title: 'Right Leg & Foot',
        instruction: 'Focus on your right toes, sole, ankle, calf, knee, and thigh. As your attention moves, tell the muscles to switch off.',
        durationSec: 90,
        targetZone: 'right-leg',
        spokenGuidance:
          'Bring all your awareness to your right foot and toes. Notice any sensations of warmth or tingling. Now draw your awareness up through your ankle, calf, knee, and right thigh. Tell those muscles to switch off and completely relax.',
      },
      {
        title: 'Left Leg & Foot',
        instruction: 'Repeat on the left leg, starting from the toes, moving up the ankle, calf, and thigh to the hip.',
        durationSec: 90,
        targetZone: 'left-leg',
        spokenGuidance:
          'Shift your awareness across to your left foot and toes. Feel the weight of your heel. Move your attention up your calf, through your knee, and into your left thigh and hip, inviting every fiber to soften.',
      },
      {
        title: 'Pelvis, Torso & Chest',
        instruction: 'Move awareness to your pelvis, lower back, stomach, and chest. Feel the gentle rise and fall of your breath.',
        durationSec: 100,
        targetZone: 'torso',
        spokenGuidance:
          'Bring your attention into your pelvis, your lower back, and your abdomen. Feel the natural rise and fall of your chest with each gentle breath. Release any tightness stored in your belly.',
      },
      {
        title: 'Arms, Hands & Shoulders',
        instruction: 'Scan both arms simultaneously, from the tops of your shoulders down past the elbows into palms and fingertips.',
        durationSec: 90,
        targetZone: 'arms',
        spokenGuidance:
          'Now scan down both arms, starting from the tops of your shoulders, down through your elbows and wrists, all the way to your palms and fingertips. Let your hands feel warm and heavy.',
      },
      {
        title: 'Neck, Jaw, Face & Crown',
        instruction: 'Bring awareness to your neck, unclench your jaw, soften your eyelids, and smooth out the skin on your forehead.',
        durationSec: 90,
        targetZone: 'head',
        spokenGuidance:
          'Finally, bring your focus up to your neck, your throat, and your jaw. Unclench your teeth. Soften the muscles around your eyes and let your forehead smooth out completely.',
      },
      {
        title: 'Complete Rest & Integration',
        instruction: 'Rest in this state of heavy, tranquil relaxation before gently wiggling your fingers and toes.',
        durationSec: 80,
        targetZone: 'full-body',
        spokenGuidance:
          'Rest in this state of profound, peaceful stillness. Feel the lightness and unity of your entire body. When you are ready, take a deeper breath and gently awaken your fingers and toes.',
      },
    ],
  },
];
